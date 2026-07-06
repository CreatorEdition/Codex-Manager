use rfd::FileDialog;

use crate::app_storage::{
    read_account_import_contents_from_directory, read_account_import_contents_from_files,
};
use crate::rpc_client::rpc_call;

const ACCOUNT_IMPORT_BATCH_BODY_LIMIT: usize = 4 * 1024 * 1024;
const ACCOUNT_IMPORT_BATCH_ITEM_LIMIT: usize = 10;
const ACCOUNT_IMPORT_RETURNED_ERROR_LIMIT: usize = 50;

fn account_import_body_size(contents: &[String]) -> Result<usize, String> {
    serde_json::to_vec(&serde_json::json!({ "contents": contents }))
        .map(|payload| payload.len())
        .map_err(|err| format!("serialize account import payload failed: {err}"))
}

fn split_account_import_batches(contents: Vec<String>) -> Result<Vec<Vec<String>>, String> {
    let mut batches = Vec::new();
    let mut batch = Vec::new();

    for content in contents {
        if batch.len() >= ACCOUNT_IMPORT_BATCH_ITEM_LIMIT {
            batches.push(std::mem::take(&mut batch));
        }

        batch.push(content);
        if account_import_body_size(&batch)? <= ACCOUNT_IMPORT_BATCH_BODY_LIMIT {
            continue;
        }
        let overflow = batch
            .pop()
            .ok_or_else(|| "账号导入批次拆分失败".to_string())?;
        if batch.is_empty() {
            return Err("单条导入内容过大，请拆分后重试".to_string());
        }
        batches.push(std::mem::take(&mut batch));
        batch.push(overflow);
        if account_import_body_size(&batch)? > ACCOUNT_IMPORT_BATCH_BODY_LIMIT {
            return Err("单条导入内容过大，请拆分后重试".to_string());
        }
    }

    if !batch.is_empty() {
        batches.push(batch);
    }
    Ok(batches)
}

fn read_import_count(payload: &serde_json::Map<String, serde_json::Value>, key: &str) -> usize {
    payload
        .get(key)
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
        .min(usize::MAX as u64) as usize
}

fn empty_import_summary() -> serde_json::Map<String, serde_json::Value> {
    let mut summary = serde_json::Map::new();
    summary.insert("canceled".to_string(), serde_json::json!(false));
    summary.insert("total".to_string(), serde_json::json!(0));
    summary.insert("created".to_string(), serde_json::json!(0));
    summary.insert("updated".to_string(), serde_json::json!(0));
    summary.insert("failed".to_string(), serde_json::json!(0));
    summary.insert("errors".to_string(), serde_json::json!([]));
    summary
}

fn account_import_result(
    response: serde_json::Value,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    if let Some(error) = response.get("error") {
        return Err(format!("account/import failed: {error}"));
    }
    response
        .get("result")
        .and_then(|value| value.as_object())
        .cloned()
        .ok_or_else(|| "account/import returned invalid response".to_string())
}

fn merge_account_import_summary(
    target: &mut serde_json::Map<String, serde_json::Value>,
    source: serde_json::Map<String, serde_json::Value>,
    index_offset: usize,
) {
    for key in ["total", "created", "updated", "failed"] {
        let merged = read_import_count(target, key).saturating_add(read_import_count(&source, key));
        target.insert(key.to_string(), serde_json::json!(merged));
    }

    let Some(source_errors) = source.get("errors").and_then(|value| value.as_array()) else {
        return;
    };
    let target_errors = target
        .entry("errors".to_string())
        .or_insert_with(|| serde_json::json!([]));
    let Some(target_errors) = target_errors.as_array_mut() else {
        return;
    };
    for error in source_errors {
        if target_errors.len() >= ACCOUNT_IMPORT_RETURNED_ERROR_LIMIT {
            break;
        }
        let Some(error) = error.as_object() else {
            continue;
        };
        let index = error
            .get("index")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            .saturating_add(index_offset as u64);
        let message = error
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        target_errors.push(serde_json::json!({
            "index": index,
            "message": message
        }));
    }
}

fn import_account_contents_in_batches(
    addr: Option<String>,
    contents: Vec<String>,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let batches = split_account_import_batches(contents)?;
    let mut summary = empty_import_summary();
    let mut processed = 0usize;

    for batch in batches {
        let batch_len = batch.len();
        let params = serde_json::json!({ "contents": batch });
        let response = rpc_call("account/import", addr.clone(), Some(params))?;
        let result = account_import_result(response)?;
        merge_account_import_summary(&mut summary, result, processed);
        processed = processed.saturating_add(batch_len);
    }

    Ok(summary)
}

/// 函数 `service_account_import`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - addr: 参数 addr
/// - contents: 参数 contents
/// - content: 参数 content
///
/// # 返回
/// 返回函数执行结果
#[tauri::command]
pub async fn service_account_import(
    addr: Option<String>,
    contents: Option<Vec<String>>,
    content: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut payload_contents = contents.unwrap_or_default();
    if let Some(single) = content {
        if !single.trim().is_empty() {
            payload_contents.push(single);
        }
    }
    tauri::async_runtime::spawn_blocking(move || {
        let result = import_account_contents_in_batches(addr, payload_contents)?;
        Ok(serde_json::json!({
            "result": result
        }))
    })
    .await
    .map_err(|err| format!("service_account_import task failed: {err}"))?
}

/// 函数 `service_account_import_by_directory`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - _addr: 参数 _addr
///
/// # 返回
/// 返回函数执行结果
#[tauri::command]
pub async fn service_account_import_by_directory(
    addr: Option<String>,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let selected_dir = FileDialog::new()
            .set_title("选择账号导入目录")
            .pick_folder();
        let Some(dir_path) = selected_dir else {
            return Ok(serde_json::json!({
              "result": {
                "ok": true,
                "canceled": true
              }
            }));
        };

        let (json_files, contents) = read_account_import_contents_from_directory(&dir_path)?;
        let mut result = import_account_contents_in_batches(addr, contents)?;
        result.insert(
            "directoryPath".to_string(),
            serde_json::json!(dir_path.to_string_lossy().to_string()),
        );
        result.insert("fileCount".to_string(), serde_json::json!(json_files.len()));
        Ok(serde_json::json!({
          "result": result
        }))
    })
    .await
    .map_err(|err| format!("service_account_import_by_directory task failed: {err}"))?
}

/// 函数 `service_account_import_by_file`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - _addr: 参数 _addr
///
/// # 返回
/// 返回函数执行结果
#[tauri::command]
pub async fn service_account_import_by_file(
    addr: Option<String>,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let selected_files = FileDialog::new()
            .set_title("选择账号导入文件")
            .add_filter("账号文件", &["json", "txt"])
            .pick_files();
        let Some(file_paths) = selected_files else {
            return Ok(serde_json::json!({
              "result": {
                "ok": true,
                "canceled": true
              }
            }));
        };

        let contents = read_account_import_contents_from_files(&file_paths)?;
        let file_count = file_paths.len();
        let mut result = import_account_contents_in_batches(addr, contents)?;
        result.insert("fileCount".to_string(), serde_json::json!(file_count));
        Ok(serde_json::json!({
          "result": result
        }))
    })
    .await
    .map_err(|err| format!("service_account_import_by_file task failed: {err}"))?
}

/// 函数 `service_account_export_by_account_files`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - addr: 参数 addr
///
/// # 返回
/// 返回函数执行结果
#[tauri::command]
pub async fn service_account_export_by_account_files(
    addr: Option<String>,
    selected_account_ids: Option<Vec<String>>,
    export_mode: Option<String>,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let selected_dir = FileDialog::new()
            .set_title("选择账号导出目录")
            .pick_folder();
        let Some(dir_path) = selected_dir else {
            return Ok(serde_json::json!({
              "result": {
                "ok": true,
                "canceled": true
              }
            }));
        };
        let params = serde_json::json!({
          "outputDir": dir_path.to_string_lossy().to_string(),
          "selectedAccountIds": selected_account_ids.unwrap_or_default(),
          "exportMode": export_mode.unwrap_or_else(|| "multiple".to_string())
        });
        rpc_call("account/export", addr, Some(params))
    })
    .await
    .map_err(|err| format!("service_account_export_by_account_files task failed: {err}"))?
}

#[cfg(test)]
mod tests {
    use super::{
        empty_import_summary, merge_account_import_summary, split_account_import_batches,
        ACCOUNT_IMPORT_BATCH_BODY_LIMIT, ACCOUNT_IMPORT_BATCH_ITEM_LIMIT,
    };

    #[test]
    fn split_account_import_batches_limits_serialized_body_size() {
        let oversized_pair_item = "x".repeat((ACCOUNT_IMPORT_BATCH_BODY_LIMIT / 2) + 4096);
        let batches =
            split_account_import_batches(vec![oversized_pair_item.clone(), oversized_pair_item])
                .expect("split batches");

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 1);
        assert_eq!(batches[1].len(), 1);
    }

    #[test]
    fn split_account_import_batches_limits_item_count() {
        let contents = (0..(ACCOUNT_IMPORT_BATCH_ITEM_LIMIT + 1))
            .map(|index| format!(r#"{{"id":"account-{index}"}}"#))
            .collect::<Vec<_>>();
        let batches = split_account_import_batches(contents).expect("split batches");

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), ACCOUNT_IMPORT_BATCH_ITEM_LIMIT);
        assert_eq!(batches[1].len(), 1);
    }

    #[test]
    fn split_account_import_batches_rejects_single_oversized_item() {
        let oversized_item = "x".repeat(ACCOUNT_IMPORT_BATCH_BODY_LIMIT + 1);

        let err = split_account_import_batches(vec![oversized_item])
            .expect_err("single oversized item should fail");

        assert!(err.contains("单条导入内容过大"));
    }

    #[test]
    fn merge_account_import_summary_offsets_batch_errors() {
        let mut target = empty_import_summary();
        let source = serde_json::json!({
            "total": 2,
            "created": 1,
            "updated": 0,
            "failed": 1,
            "errors": [
                { "index": 1, "message": "bad account" }
            ]
        })
        .as_object()
        .cloned()
        .expect("source object");

        merge_account_import_summary(&mut target, source, 10);

        assert_eq!(
            target.get("total").and_then(|value| value.as_u64()),
            Some(2)
        );
        assert_eq!(
            target.get("created").and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            target.get("failed").and_then(|value| value.as_u64()),
            Some(1)
        );
        let errors = target
            .get("errors")
            .and_then(|value| value.as_array())
            .expect("errors");
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].get("index").and_then(|value| value.as_u64()),
            Some(11)
        );
    }
}
