use super::*;

#[test]
fn validate_background_tasks_settings_rejects_invalid_warmup_cron_expression() {
    let patch = BackgroundTasksSettingsPatch {
        warmup_cron_expression: Some("99 99 99 99 99".to_string()),
        ..BackgroundTasksSettingsPatch::default()
    };
    let err = validate_background_tasks_settings_patch(&patch)
        .expect_err("invalid cron should be rejected");

    assert!(!err.trim().is_empty());
}

#[test]
fn validate_background_tasks_settings_rejects_enabled_empty_warmup_cron_expression() {
    let patch = BackgroundTasksSettingsPatch {
        warmup_cron_enabled: Some(true),
        warmup_cron_expression: Some("   ".to_string()),
        ..BackgroundTasksSettingsPatch::default()
    };
    let err = validate_background_tasks_settings_patch(&patch)
        .expect_err("enabled empty cron should be rejected");

    assert!(err.contains("required"));
}
