/// 归一化日志中的 URL，避免 query-secret 等敏感查询参数进入 DB、UI 或磁盘日志。
pub(crate) fn redact_url_for_log(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Ok(mut url) = reqwest::Url::parse(trimmed) {
        url.set_query(None);
        url.set_fragment(None);
        return url.to_string();
    }

    let end = trimmed.find(['?', '#']).unwrap_or(trimmed.len());
    trimmed[..end].to_string()
}

pub(crate) fn normalize_optional_url_for_log(raw: Option<&str>) -> Option<String> {
    raw.map(redact_url_for_log)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::redact_url_for_log;

    #[test]
    fn redacts_query_and_fragment_from_absolute_urls() {
        assert_eq!(
            redact_url_for_log(" https://gateway.example.com/v1/responses?api_key=secret#frag "),
            "https://gateway.example.com/v1/responses"
        );
    }

    #[test]
    fn redacts_query_from_relative_paths() {
        assert_eq!(
            redact_url_for_log("/v1/responses?token=secret"),
            "/v1/responses"
        );
    }
}
