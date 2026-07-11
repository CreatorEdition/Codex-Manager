use codexmanager_core::storage::now_ts;
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::Value;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

const ENV_ENABLED: &str = "CODEXMANAGER_IP_DIAGNOSTICS_ENABLED";
const ENV_CACHE_TTL_SECS: &str = "CODEXMANAGER_IP_DIAGNOSTICS_CACHE_TTL_SECS";
const ENV_REGION_THROTTLE_SECS: &str = "CODEXMANAGER_IP_DIAGNOSTICS_REGION_THROTTLE_SECS";
const ENV_BLOCKED_INTERVAL_SECS: &str = "CODEXMANAGER_IP_DIAGNOSTICS_BLOCKED_INTERVAL_SECS";
const ENV_SOURCE_TIMEOUT_MS: &str = "CODEXMANAGER_IP_DIAGNOSTICS_SOURCE_TIMEOUT_MS";
const ENV_TOTAL_TIMEOUT_MS: &str = "CODEXMANAGER_IP_DIAGNOSTICS_TOTAL_TIMEOUT_MS";

const DEFAULT_CACHE_TTL_SECS: u64 = 1_800;
const DEFAULT_REGION_THROTTLE_SECS: u64 = 300;
const DEFAULT_BLOCKED_INTERVAL_SECS: u64 = 1_800;
const DEFAULT_SOURCE_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_TOTAL_TIMEOUT_MS: u64 = 12_000;
const MANUAL_REFRESH_MIN_INTERVAL_SECS: i64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IpServiceKind {
    IpSb,
    IpApiCo,
    IpWhoIs,
    GeoJs,
}

#[derive(Debug, Clone, Copy)]
struct IpService {
    name: &'static str,
    url: &'static str,
    kind: IpServiceKind,
}

const IP_SERVICES: &[IpService] = &[
    IpService {
        name: "ip_sb",
        url: "https://api.ip.sb/geoip",
        kind: IpServiceKind::IpSb,
    },
    IpService {
        name: "ipapi_co",
        url: "https://ipapi.co/json",
        kind: IpServiceKind::IpApiCo,
    },
    IpService {
        name: "ipwho_is",
        url: "https://ipwho.is/",
        kind: IpServiceKind::IpWhoIs,
    },
    IpService {
        name: "geojs",
        url: "https://get.geojs.io/v1/ip/geo.json",
        kind: IpServiceKind::GeoJs,
    },
];

#[derive(Debug, Clone)]
struct NetworkDiagnosticRecord {
    ip: String,
    country_code: Option<String>,
    country: Option<String>,
    asn: Option<u64>,
    organization: Option<String>,
    checked_at: i64,
    source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDiagnosticsView {
    enabled: bool,
    refreshing: bool,
    refresh_scheduled: bool,
    ip: Option<String>,
    country_code: Option<String>,
    country: Option<String>,
    asn: Option<u64>,
    organization: Option<String>,
    checked_at: Option<i64>,
    last_attempt_at: Option<i64>,
    source: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Default)]
struct DiagnosticsInner {
    last_success: Option<NetworkDiagnosticRecord>,
    last_error: Option<String>,
    last_attempt_at: Option<i64>,
    last_region_trigger_at: Option<i64>,
    next_source_index: usize,
    refreshing: bool,
}

#[derive(Debug, Default)]
struct DiagnosticsState {
    inner: Mutex<DiagnosticsInner>,
}

#[derive(Debug, Clone, Copy)]
enum RefreshReason {
    Startup,
    CacheStale,
    RegionBlocked,
    BlockedFallback,
    Manual,
}

impl RefreshReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::CacheStale => "cache_stale",
            Self::RegionBlocked => "region_blocked",
            Self::BlockedFallback => "blocked_fallback",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DiagnosticsConfig {
    enabled: bool,
    cache_ttl_secs: u64,
    region_throttle_secs: u64,
    blocked_interval_secs: u64,
    source_timeout_ms: u64,
    total_timeout_ms: u64,
}

#[derive(Debug)]
struct AttemptError {
    message: String,
    retryable: bool,
}

static DIAGNOSTICS_STATE: OnceLock<Arc<DiagnosticsState>> = OnceLock::new();
static FALLBACK_SCHEDULER_STARTED: AtomicBool = AtomicBool::new(false);

fn diagnostics_state() -> Arc<DiagnosticsState> {
    DIAGNOSTICS_STATE
        .get_or_init(|| Arc::new(DiagnosticsState::default()))
        .clone()
}

/// 启动出口网络诊断后台任务。
///
/// 启动检查和阻断兜底都在独立线程执行，不阻塞服务监听与网关请求路径。
pub(crate) fn ensure_network_diagnostics() {
    let config = diagnostics_config();
    if !config.enabled {
        return;
    }
    let _ = request_refresh(RefreshReason::Startup);
    if FALLBACK_SCHEDULER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    if let Err(err) = std::thread::Builder::new()
        .name("ip-diagnostics-fallback".to_string())
        .spawn(move || loop {
            let interval = diagnostics_config().blocked_interval_secs;
            std::thread::sleep(Duration::from_secs(interval));
            if crate::shutdown_requested() {
                break;
            }
            if has_region_blocked_accounts() {
                let _ = request_refresh(RefreshReason::BlockedFallback);
            }
        })
    {
        FALLBACK_SCHEDULER_STARTED.store(false, Ordering::SeqCst);
        log::warn!("event=ip_diagnostics_scheduler_start_failed err={err}");
    }
}

/// 在上游明确返回区域阻断后，节流触发一次出口诊断。
///
/// 本函数只收集诊断信息，绝不修改账号状态。
pub(crate) fn notify_region_blocked() {
    let _ = request_refresh(RefreshReason::RegionBlocked);
}

/// 读取当前出口诊断快照，并在缓存过期时异步补刷。
pub fn network_diagnostics_get() -> NetworkDiagnosticsView {
    let scheduled = request_refresh(RefreshReason::CacheStale);
    build_view(scheduled)
}

/// 请求手动刷新出口诊断，立即返回当前快照和调度状态。
pub fn network_diagnostics_refresh() -> NetworkDiagnosticsView {
    let scheduled = request_refresh(RefreshReason::Manual);
    build_view(scheduled)
}

fn request_refresh(reason: RefreshReason) -> bool {
    let config = diagnostics_config();
    if !config.enabled {
        return false;
    }
    let state = diagnostics_state();
    let now = now_ts();
    let start_index = {
        let mut inner = crate::lock_utils::lock_recover(&state.inner, "ip_diagnostics_state");
        if inner.refreshing || !refresh_due(&inner, reason, now, config) {
            return false;
        }
        inner.refreshing = true;
        inner.last_attempt_at = Some(now);
        if matches!(reason, RefreshReason::RegionBlocked) {
            inner.last_region_trigger_at = Some(now);
        }
        let start_index = inner.next_source_index % IP_SERVICES.len();
        inner.next_source_index = (start_index + 1) % IP_SERVICES.len();
        start_index
    };

    let state_for_worker = Arc::clone(&state);
    let spawn_result = std::thread::Builder::new()
        .name("ip-diagnostics-query".to_string())
        .spawn(move || {
            let result = query_network_diagnostics(start_index, config);
            let mut inner =
                crate::lock_utils::lock_recover(&state_for_worker.inner, "ip_diagnostics_state");
            inner.refreshing = false;
            match result {
                Ok(record) => {
                    log::info!(
                        "event=ip_diagnostics_succeeded reason={} source={} country_code={} asn={}",
                        reason.as_str(),
                        record.source,
                        record.country_code.as_deref().unwrap_or("unknown"),
                        record.asn.unwrap_or_default()
                    );
                    inner.last_success = Some(record);
                    inner.last_error = None;
                }
                Err(err) => {
                    log::warn!(
                        "event=ip_diagnostics_failed reason={} err={}",
                        reason.as_str(),
                        err
                    );
                    inner.last_error = Some(err);
                }
            }
        });

    if let Err(err) = spawn_result {
        let mut inner = crate::lock_utils::lock_recover(&state.inner, "ip_diagnostics_state");
        inner.refreshing = false;
        inner.last_error = Some("无法启动出口诊断后台任务".to_string());
        log::warn!("event=ip_diagnostics_worker_start_failed err={err}");
        return false;
    }
    true
}

fn refresh_due(
    inner: &DiagnosticsInner,
    reason: RefreshReason,
    now: i64,
    config: DiagnosticsConfig,
) -> bool {
    let elapsed_since_attempt = inner
        .last_attempt_at
        .map(|last| now.saturating_sub(last))
        .unwrap_or(i64::MAX);
    match reason {
        RefreshReason::Startup => inner.last_attempt_at.is_none(),
        RefreshReason::Manual => elapsed_since_attempt >= MANUAL_REFRESH_MIN_INTERVAL_SECS,
        RefreshReason::RegionBlocked => inner
            .last_region_trigger_at
            .map(|last| now.saturating_sub(last) >= config.region_throttle_secs as i64)
            .unwrap_or(true),
        RefreshReason::CacheStale => elapsed_since_attempt >= config.cache_ttl_secs as i64,
        RefreshReason::BlockedFallback => {
            elapsed_since_attempt >= config.blocked_interval_secs as i64
        }
    }
}

fn build_view(refresh_scheduled: bool) -> NetworkDiagnosticsView {
    let config = diagnostics_config();
    let state = diagnostics_state();
    let inner = crate::lock_utils::lock_recover(&state.inner, "ip_diagnostics_state");
    let record = inner.last_success.as_ref();
    NetworkDiagnosticsView {
        enabled: config.enabled,
        refreshing: inner.refreshing,
        refresh_scheduled,
        ip: record.map(|value| value.ip.clone()),
        country_code: record.and_then(|value| value.country_code.clone()),
        country: record.and_then(|value| value.country.clone()),
        asn: record.and_then(|value| value.asn),
        organization: record.and_then(|value| value.organization.clone()),
        checked_at: record.map(|value| value.checked_at),
        last_attempt_at: inner.last_attempt_at,
        source: record.map(|value| value.source.clone()),
        error: inner.last_error.clone(),
    }
}

fn query_network_diagnostics(
    start_index: usize,
    config: DiagnosticsConfig,
) -> Result<NetworkDiagnosticRecord, String> {
    let client = crate::gateway::fresh_upstream_client();
    let started_at = Instant::now();
    let total_timeout = Duration::from_millis(config.total_timeout_ms);
    let mut last_error = "没有可用的出口诊断服务".to_string();

    for offset in 0..IP_SERVICES.len() {
        let service = IP_SERVICES[(start_index + offset) % IP_SERVICES.len()];
        for attempt in 0..=1 {
            let Some(remaining) = total_timeout.checked_sub(started_at.elapsed()) else {
                return Err("出口诊断超过总超时".to_string());
            };
            if remaining < Duration::from_millis(100) {
                return Err("出口诊断超过总超时".to_string());
            }
            let request_timeout = remaining.min(Duration::from_millis(config.source_timeout_ms));
            match query_service(&client, service, request_timeout) {
                Ok(record) => return Ok(record),
                Err(err) => {
                    last_error = format!("{}: {}", service.name, err.message);
                    if !err.retryable || attempt > 0 {
                        break;
                    }
                }
            }
        }
    }
    Err(format!("所有出口诊断服务均失败（{last_error}）"))
}

fn query_service(
    client: &Client,
    service: IpService,
    timeout: Duration,
) -> Result<NetworkDiagnosticRecord, AttemptError> {
    let response = client
        .get(service.url)
        .header("accept", "application/json")
        .header("user-agent", crate::gateway::current_codex_user_agent())
        .timeout(timeout)
        .send()
        .map_err(|err| AttemptError {
            message: if err.is_timeout() {
                "请求超时".to_string()
            } else if err.is_connect() {
                "连接失败".to_string()
            } else {
                "网络错误".to_string()
            },
            retryable: true,
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(AttemptError {
            message: format!("HTTP {}", status.as_u16()),
            retryable: status.is_server_error(),
        });
    }
    let payload = response.json::<Value>().map_err(|_| AttemptError {
        message: "响应不是有效 JSON".to_string(),
        retryable: false,
    })?;
    map_service_response(service, &payload).map_err(|message| AttemptError {
        message,
        retryable: false,
    })
}

fn map_service_response(
    service: IpService,
    payload: &Value,
) -> Result<NetworkDiagnosticRecord, String> {
    let (ip, country_code, country, asn, organization) = match service.kind {
        IpServiceKind::IpSb => (
            text_at(payload, &["ip"]),
            text_at(payload, &["country_code"]),
            text_at(payload, &["country"]),
            asn_at(payload, &["asn"]),
            first_text(
                payload,
                &[&["asn_organization"], &["organization"], &["isp"]],
            ),
        ),
        IpServiceKind::IpApiCo => (
            text_at(payload, &["ip"]),
            text_at(payload, &["country_code"]),
            text_at(payload, &["country_name"]),
            asn_at(payload, &["asn"]),
            text_at(payload, &["org"]),
        ),
        IpServiceKind::IpWhoIs => (
            text_at(payload, &["ip"]),
            text_at(payload, &["country_code"]),
            text_at(payload, &["country"]),
            asn_at(payload, &["connection", "asn"]),
            first_text(payload, &[&["connection", "org"], &["connection", "isp"]]),
        ),
        IpServiceKind::GeoJs => (
            text_at(payload, &["ip"]),
            text_at(payload, &["country_code"]),
            text_at(payload, &["country"]),
            asn_at(payload, &["asn"]),
            text_at(payload, &["organization_name"]),
        ),
    };
    let ip = ip.ok_or_else(|| "响应缺少 IP".to_string())?;
    if ip.parse::<IpAddr>().is_err() {
        return Err("响应包含无效 IP".to_string());
    }
    Ok(NetworkDiagnosticRecord {
        ip,
        country_code: normalize_country_code(country_code),
        country: normalize_text(country, 80),
        asn,
        organization: normalize_text(organization, 160),
        checked_at: now_ts(),
        source: service.name.to_string(),
    })
}

fn text_at(payload: &Value, path: &[&str]) -> Option<String> {
    let mut current = payload;
    for segment in path {
        current = current.get(*segment)?;
    }
    match current {
        Value::String(value) => normalize_text(Some(value.clone()), 256),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn first_text(payload: &Value, paths: &[&[&str]]) -> Option<String> {
    paths.iter().find_map(|path| text_at(payload, path))
}

fn asn_at(payload: &Value, path: &[&str]) -> Option<u64> {
    let value = text_at(payload, path)?;
    let normalized = value.trim().trim_start_matches(|character: char| {
        character == 'A' || character == 'a' || character == 'S' || character == 's'
    });
    normalized.parse::<u64>().ok().filter(|asn| *asn > 0)
}

fn normalize_country_code(value: Option<String>) -> Option<String> {
    let normalized = value?.trim().to_ascii_uppercase();
    if normalized.len() == 2 && normalized.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        Some(normalized)
    } else {
        None
    }
}

fn normalize_text(value: Option<String>, max_chars: usize) -> Option<String> {
    let trimmed = value?.trim().to_string();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.chars().take(max_chars).collect())
}

fn has_region_blocked_accounts() -> bool {
    let Some(storage) = crate::storage_helpers::open_storage() else {
        return false;
    };
    let Ok(accounts) = storage.list_accounts() else {
        return false;
    };
    let account_ids = accounts
        .into_iter()
        .filter(|account| account.status.trim().eq_ignore_ascii_case("unavailable"))
        .map(|account| account.id)
        .collect::<Vec<_>>();
    if account_ids.is_empty() {
        return false;
    }
    storage
        .latest_account_status_reasons(&account_ids)
        .map(|reasons| {
            reasons.values().any(|reason| {
                reason.trim() == crate::account_status::REFRESH_TOKEN_REGION_BLOCKED_REASON
            })
        })
        .unwrap_or(false)
}

fn diagnostics_config() -> DiagnosticsConfig {
    DiagnosticsConfig {
        enabled: env_bool(ENV_ENABLED, !cfg!(test)),
        cache_ttl_secs: env_u64(ENV_CACHE_TTL_SECS, DEFAULT_CACHE_TTL_SECS, 60, 86_400),
        region_throttle_secs: env_u64(
            ENV_REGION_THROTTLE_SECS,
            DEFAULT_REGION_THROTTLE_SECS,
            30,
            3_600,
        ),
        blocked_interval_secs: env_u64(
            ENV_BLOCKED_INTERVAL_SECS,
            DEFAULT_BLOCKED_INTERVAL_SECS,
            300,
            21_600,
        ),
        source_timeout_ms: env_u64(
            ENV_SOURCE_TIMEOUT_MS,
            DEFAULT_SOURCE_TIMEOUT_MS,
            1_000,
            10_000,
        ),
        total_timeout_ms: env_u64(
            ENV_TOTAL_TIMEOUT_MS,
            DEFAULT_TOTAL_TIMEOUT_MS,
            2_000,
            30_000,
        ),
    }
}

fn env_bool(key: &str, fallback: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => fallback,
        })
        .unwrap_or(fallback)
}

fn env_u64(key: &str, fallback: u64, min: u64, max: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(fallback)
        .clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_service_specific_fields_without_location_details() {
        let cases = [
            (
                IP_SERVICES[0],
                json!({"ip":"203.0.113.1","country_code":"us","country":"United States","asn":64500,"asn_organization":"Example ASN","latitude":1.2,"longitude":3.4}),
            ),
            (
                IP_SERVICES[1],
                json!({"ip":"2001:db8::1","country_code":"jp","country_name":"Japan","asn":"AS64501","org":"Example Org"}),
            ),
            (
                IP_SERVICES[2],
                json!({"ip":"198.51.100.2","country_code":"de","country":"Germany","connection":{"asn":64502,"org":"Example ISP"}}),
            ),
            (
                IP_SERVICES[3],
                json!({"ip":"192.0.2.3","country_code":"fr","country":"France","asn":"64503","organization_name":"Example Network"}),
            ),
        ];
        for (service, payload) in cases {
            let record = map_service_response(service, &payload).expect("map response");
            assert_eq!(record.country_code.as_deref().map(str::len), Some(2));
            assert!(record.asn.unwrap_or_default() >= 64_500);
            assert!(record.organization.is_some());
        }
    }

    #[test]
    fn rejects_invalid_ip_and_country_code() {
        let invalid_ip = map_service_response(
            IP_SERVICES[0],
            &json!({"ip":"not-an-ip","country_code":"USA"}),
        );
        assert_eq!(invalid_ip.unwrap_err(), "响应包含无效 IP");
        let record = map_service_response(
            IP_SERVICES[0],
            &json!({"ip":"203.0.113.9","country_code":"USA"}),
        )
        .expect("valid record");
        assert_eq!(record.country_code, None);
    }

    #[test]
    fn refresh_due_applies_cache_manual_and_region_throttles() {
        let config = DiagnosticsConfig {
            enabled: true,
            cache_ttl_secs: 1_000,
            region_throttle_secs: 30,
            blocked_interval_secs: 300,
            source_timeout_ms: 5_000,
            total_timeout_ms: 12_000,
        };
        let inner = DiagnosticsInner {
            last_attempt_at: Some(1_000),
            last_region_trigger_at: Some(990),
            ..DiagnosticsInner::default()
        };
        assert!(!refresh_due(
            &inner,
            RefreshReason::CacheStale,
            1_050,
            config
        ));
        assert!(refresh_due(
            &inner,
            RefreshReason::CacheStale,
            2_000,
            config
        ));
        assert!(!refresh_due(&inner, RefreshReason::Manual, 1_004, config));
        assert!(refresh_due(&inner, RefreshReason::Manual, 1_005, config));
        assert!(!refresh_due(
            &inner,
            RefreshReason::RegionBlocked,
            1_019,
            config
        ));
        assert!(refresh_due(
            &inner,
            RefreshReason::RegionBlocked,
            1_020,
            config
        ));
        assert!(!refresh_due(
            &inner,
            RefreshReason::BlockedFallback,
            1_299,
            config
        ));
        assert!(refresh_due(
            &inner,
            RefreshReason::BlockedFallback,
            1_300,
            config
        ));
    }

    #[test]
    fn service_rotation_uses_each_source_once_per_cycle() {
        for start_index in 0..IP_SERVICES.len() {
            let ordered = (0..IP_SERVICES.len())
                .map(|offset| IP_SERVICES[(start_index + offset) % IP_SERVICES.len()].name)
                .collect::<Vec<_>>();
            let mut unique = ordered.clone();
            unique.sort_unstable();
            unique.dedup();
            assert_eq!(unique.len(), IP_SERVICES.len());
            assert_eq!(ordered[0], IP_SERVICES[start_index].name);
        }
    }

    #[test]
    fn http_retry_policy_only_retries_server_errors() {
        assert!(reqwest::StatusCode::BAD_GATEWAY.is_server_error());
        assert!(!reqwest::StatusCode::TOO_MANY_REQUESTS.is_server_error());
        assert!(!reqwest::StatusCode::FORBIDDEN.is_server_error());
    }
}
