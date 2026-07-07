use super::{Arc, Mutex, UpstreamResponseUsage};
use std::io::{BufRead, BufReader, Read};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_SSE_KEEPALIVE_INTERVAL_MS: u64 = 15_000;
const ENV_SSE_KEEPALIVE_INTERVAL_MS: &str = "CODEXMANAGER_SSE_KEEPALIVE_INTERVAL_MS";
const UPSTREAM_SSE_FRAME_CHANNEL_CAPACITY: usize = 128;

static SSE_KEEPALIVE_INTERVAL_MS: AtomicU64 = AtomicU64::new(DEFAULT_SSE_KEEPALIVE_INTERVAL_MS);
const STREAM_INCOMPLETE_FALLBACK_MESSAGE: &str = "连接中断（可能是网络波动或客户端主动取消）";
const STREAM_READ_FAILED_FALLBACK_MESSAGE: &str = "上游中途断开，未返回具体错误信息";
const STREAM_IDLE_TIMEOUT_FALLBACK_MESSAGE: &str = "上游流式空闲超时";
const PASSTHROUGH_SSE_COLLECTOR_LOCK: &str = "http_bridge_passthrough_sse_collector";
const UPSTREAM_RESPONSE_USAGE_LOCK: &str = "http_bridge_upstream_response_usage";

#[derive(Debug, Clone, Default)]
pub(crate) struct PassthroughSseCollector {
    pub(crate) usage: UpstreamResponseUsage,
    pub(crate) saw_terminal: bool,
    pub(crate) terminal_error: Option<String>,
    pub(crate) upstream_error_hint: Option<String>,
    pub(crate) last_event_type: Option<String>,
}

fn elapsed_ms_since(started_at: Instant) -> i64 {
    started_at.elapsed().as_millis().min(i64::MAX as u128) as i64
}

pub(super) fn with_passthrough_collector<R>(
    usage_collector: &Arc<Mutex<PassthroughSseCollector>>,
    update: impl FnOnce(&mut PassthroughSseCollector) -> R,
) -> R {
    let mut collector =
        crate::lock_utils::lock_recover(usage_collector.as_ref(), PASSTHROUGH_SSE_COLLECTOR_LOCK);
    update(&mut collector)
}

pub(super) fn with_usage_collector<R>(
    usage_collector: &Arc<Mutex<UpstreamResponseUsage>>,
    update: impl FnOnce(&mut UpstreamResponseUsage) -> R,
) -> R {
    let mut usage =
        crate::lock_utils::lock_recover(usage_collector.as_ref(), UPSTREAM_RESPONSE_USAGE_LOCK);
    update(&mut usage)
}

pub(super) fn snapshot_passthrough_collector(
    usage_collector: &Arc<Mutex<PassthroughSseCollector>>,
) -> PassthroughSseCollector {
    with_passthrough_collector(usage_collector, |collector| collector.clone())
}

pub(super) fn snapshot_usage_collector(
    usage_collector: &Arc<Mutex<UpstreamResponseUsage>>,
) -> UpstreamResponseUsage {
    with_usage_collector(usage_collector, |usage| usage.clone())
}

pub(super) fn mark_first_response_ms(
    usage_collector: &Arc<Mutex<PassthroughSseCollector>>,
    started_at: Instant,
) {
    with_passthrough_collector(usage_collector, |collector| {
        if collector.usage.first_response_ms.is_none() {
            collector.usage.first_response_ms = Some(elapsed_ms_since(started_at));
        }
    });
}

pub(super) fn mark_first_response_ms_on_usage(
    usage_collector: &Arc<Mutex<UpstreamResponseUsage>>,
    started_at: Instant,
) {
    with_usage_collector(usage_collector, |usage| {
        if usage.first_response_ms.is_none() {
            usage.first_response_ms = Some(elapsed_ms_since(started_at));
        }
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SseKeepAliveFrame {
    Comment,
    OpenAIResponses,
    Anthropic,
}

impl SseKeepAliveFrame {
    /// 函数 `bytes`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - crate: 参数 crate
    ///
    /// # 返回
    /// 返回函数执行结果
    pub(crate) fn bytes(self) -> &'static [u8] {
        match self {
            Self::Comment => b": keep-alive\n\n",
            Self::OpenAIResponses => b"data: {\"type\":\"codexmanager.keepalive\"}\n\n",
            Self::Anthropic => b"event: ping\ndata: {\"type\":\"ping\"}\n\n",
        }
    }
}

#[derive(Debug)]
pub(crate) enum UpstreamSseFramePumpItem {
    Frame(Vec<String>),
    Eof,
    Error(String),
}

pub(crate) struct UpstreamSseFramePump {
    rx: Receiver<UpstreamSseFramePumpItem>,
}

impl UpstreamSseFramePump {
    pub(crate) fn from_reader<R>(upstream: R) -> Self
    where
        R: Read + Send + 'static,
    {
        let (tx, rx) =
            mpsc::sync_channel::<UpstreamSseFramePumpItem>(UPSTREAM_SSE_FRAME_CHANNEL_CAPACITY);
        thread::spawn(move || {
            let mut reader = BufReader::new(upstream);
            let mut pending_frame_lines = Vec::new();
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        if !pending_frame_lines.is_empty()
                            && tx
                                .send(UpstreamSseFramePumpItem::Frame(pending_frame_lines))
                                .is_err()
                        {
                            return;
                        }
                        let _ = tx.send(UpstreamSseFramePumpItem::Eof);
                        return;
                    }
                    Ok(_) => {
                        let is_blank = line == "\n" || line == "\r\n";
                        pending_frame_lines.push(line);
                        if is_blank {
                            let frame = std::mem::take(&mut pending_frame_lines);
                            if tx.send(UpstreamSseFramePumpItem::Frame(frame)).is_err() {
                                return;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(UpstreamSseFramePumpItem::Error(err.to_string()));
                        return;
                    }
                }
            }
        });
        Self { rx }
    }

    /// 函数 `new`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - crate: 参数 crate
    ///
    /// # 返回
    /// 返回函数执行结果
    pub(crate) fn new(upstream: reqwest::blocking::Response) -> Self {
        Self::from_reader(upstream)
    }

    /// 函数 `recv_timeout`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - crate: 参数 crate
    ///
    /// # 返回
    /// 返回函数执行结果
    pub(crate) fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<UpstreamSseFramePumpItem, RecvTimeoutError> {
        self.rx.recv_timeout(timeout)
    }
}

/// 函数 `reload_from_env`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 无
pub(super) fn reload_from_env() {
    SSE_KEEPALIVE_INTERVAL_MS.store(
        std::env::var(ENV_SSE_KEEPALIVE_INTERVAL_MS)
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_SSE_KEEPALIVE_INTERVAL_MS),
        Ordering::Relaxed,
    );
}

/// 函数 `sse_keepalive_interval`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 返回函数执行结果
pub(super) fn sse_keepalive_interval() -> Duration {
    let interval_ms = SSE_KEEPALIVE_INTERVAL_MS.load(Ordering::Relaxed);
    Duration::from_millis(interval_ms.max(1))
}

pub(super) fn stream_wait_timeout(last_upstream_activity: Instant) -> Duration {
    let keepalive = sse_keepalive_interval();
    let Some(idle_timeout) = crate::gateway::upstream_stream_timeout() else {
        return keepalive;
    };
    let elapsed = last_upstream_activity.elapsed();
    if elapsed >= idle_timeout {
        return Duration::from_millis(1);
    }
    keepalive.min(
        idle_timeout
            .saturating_sub(elapsed)
            .max(Duration::from_millis(1)),
    )
}

pub(super) fn stream_idle_timed_out(last_upstream_activity: Instant) -> bool {
    crate::gateway::upstream_stream_timeout()
        .is_some_and(|idle_timeout| last_upstream_activity.elapsed() >= idle_timeout)
}

pub(super) fn stream_idle_timeout_message() -> String {
    STREAM_IDLE_TIMEOUT_FALLBACK_MESSAGE.to_string()
}

pub(super) fn should_emit_keepalive(saw_upstream_frame: bool) -> bool {
    saw_upstream_frame
}

/// 函数 `current_sse_keepalive_interval_ms`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 返回函数执行结果
pub(super) fn current_sse_keepalive_interval_ms() -> u64 {
    SSE_KEEPALIVE_INTERVAL_MS.load(Ordering::Relaxed).max(1)
}

/// 函数 `set_sse_keepalive_interval_ms`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 返回函数执行结果
pub(super) fn set_sse_keepalive_interval_ms(interval_ms: u64) -> Result<u64, String> {
    if interval_ms == 0 {
        return Err("SSE keepalive interval must be greater than 0".to_string());
    }
    SSE_KEEPALIVE_INTERVAL_MS.store(interval_ms, Ordering::Relaxed);
    std::env::set_var(ENV_SSE_KEEPALIVE_INTERVAL_MS, interval_ms.to_string());
    Ok(interval_ms)
}

/// 函数 `mark_collector_terminal_success`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 无
pub(super) fn mark_collector_terminal_success(
    usage_collector: &Arc<Mutex<PassthroughSseCollector>>,
) {
    with_passthrough_collector(usage_collector, |collector| {
        collector.saw_terminal = true;
        collector.terminal_error = None;
    });
}

/// 函数 `stream_incomplete_message`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 返回函数执行结果
pub(super) fn stream_incomplete_message() -> String {
    STREAM_INCOMPLETE_FALLBACK_MESSAGE.to_string()
}

/// 函数 `stream_reader_disconnected_message`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 返回函数执行结果
pub(super) fn stream_reader_disconnected_message() -> String {
    STREAM_INCOMPLETE_FALLBACK_MESSAGE.to_string()
}

pub(super) fn upstream_hint_or_stream_incomplete_message(
    upstream_error_hint: Option<&str>,
) -> String {
    upstream_error_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(stream_incomplete_message)
}

/// 函数 `classify_upstream_stream_read_error`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 返回函数执行结果
pub(super) fn classify_upstream_stream_read_error(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return STREAM_READ_FAILED_FALLBACK_MESSAGE.to_string();
    }
    let normalized = trimmed.to_ascii_lowercase();
    if normalized == "request or response body error" || normalized == "stream read failed" {
        return STREAM_READ_FAILED_FALLBACK_MESSAGE.to_string();
    }
    if normalized.contains("timed out") || normalized.contains("timeout") {
        return STREAM_IDLE_TIMEOUT_FALLBACK_MESSAGE.to_string();
    }
    if normalized.contains("connection reset")
        || normalized.contains("connection aborted")
        || normalized.contains("connection was forcibly closed")
        || normalized.contains("broken pipe")
        || normalized.contains("unexpected eof")
        || normalized.contains("connection closed before message completed")
    {
        return STREAM_INCOMPLETE_FALLBACK_MESSAGE.to_string();
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        classify_upstream_stream_read_error, mark_collector_terminal_success,
        mark_first_response_ms, mark_first_response_ms_on_usage, snapshot_passthrough_collector,
        snapshot_usage_collector, stream_incomplete_message, stream_reader_disconnected_message,
        with_passthrough_collector, UpstreamResponseUsage,
    };
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    fn poison_passthrough_collector() -> Arc<Mutex<super::PassthroughSseCollector>> {
        let collector = Arc::new(Mutex::new(super::PassthroughSseCollector::default()));
        let cloned = Arc::clone(&collector);
        let _ = std::thread::spawn(move || {
            let _guard = cloned.lock().expect("collector lock");
            panic!("poison passthrough collector for test");
        })
        .join();
        assert!(collector.is_poisoned());
        collector
    }

    fn poison_usage_collector() -> Arc<Mutex<UpstreamResponseUsage>> {
        let usage = Arc::new(Mutex::new(UpstreamResponseUsage::default()));
        let cloned = Arc::clone(&usage);
        let _ = std::thread::spawn(move || {
            let _guard = cloned.lock().expect("usage lock");
            panic!("poison usage collector for test");
        })
        .join();
        assert!(usage.is_poisoned());
        usage
    }

    /// 函数 `classify_upstream_stream_read_error_maps_body_error`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// 无
    ///
    /// # 返回
    /// 无
    #[test]
    fn classify_upstream_stream_read_error_maps_body_error() {
        assert_eq!(
            classify_upstream_stream_read_error("request or response body error"),
            "上游中途断开，未返回具体错误信息"
        );
    }

    /// 函数 `classify_upstream_stream_read_error_maps_disconnect`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// 无
    ///
    /// # 返回
    /// 无
    #[test]
    fn classify_upstream_stream_read_error_maps_disconnect() {
        assert_eq!(
            classify_upstream_stream_read_error("connection reset by peer"),
            "连接中断（可能是网络波动或客户端主动取消）"
        );
    }

    /// 函数 `classify_upstream_stream_read_error_maps_timeout`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// 无
    ///
    /// # 返回
    /// 无
    #[test]
    fn classify_upstream_stream_read_error_maps_timeout() {
        assert_eq!(
            classify_upstream_stream_read_error("operation timed out"),
            "上游流式空闲超时"
        );
    }

    /// 函数 `stream_terminal_messages_are_user_friendly`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// 无
    ///
    /// # 返回
    /// 无
    #[test]
    fn stream_terminal_messages_are_user_friendly() {
        assert_eq!(
            stream_incomplete_message(),
            "连接中断（可能是网络波动或客户端主动取消）"
        );
        assert_eq!(
            stream_reader_disconnected_message(),
            "连接中断（可能是网络波动或客户端主动取消）"
        );
        assert_eq!(super::stream_idle_timeout_message(), "上游流式空闲超时");
    }

    /// 函数 `passthrough_collector_recovers_from_poisoned_lock`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// 无
    ///
    /// # 返回
    /// 无
    #[test]
    fn passthrough_collector_recovers_from_poisoned_lock() {
        let collector = poison_passthrough_collector();

        mark_first_response_ms(&collector, Instant::now());
        with_passthrough_collector(&collector, |collector| {
            collector.terminal_error = Some("upstream failed".to_string());
        });
        mark_collector_terminal_success(&collector);

        let snapshot = snapshot_passthrough_collector(&collector);
        assert!(snapshot.usage.first_response_ms.is_some());
        assert!(snapshot.saw_terminal);
        assert_eq!(snapshot.terminal_error, None);
    }

    /// 函数 `usage_collector_recovers_from_poisoned_lock`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// 无
    ///
    /// # 返回
    /// 无
    #[test]
    fn usage_collector_recovers_from_poisoned_lock() {
        let usage = poison_usage_collector();

        mark_first_response_ms_on_usage(&usage, Instant::now());
        let snapshot = snapshot_usage_collector(&usage);

        assert!(snapshot.first_response_ms.is_some());
    }
}
