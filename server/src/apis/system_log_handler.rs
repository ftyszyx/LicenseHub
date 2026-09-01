use crate::apis::list_api::ListParamsReq;
use crate::core::app::{LOG_DIRECTORY, LOG_FILE_PREFIX};
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::{NaiveDate, NaiveDateTime};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;

const MAX_KEYWORD_CHARS: usize = 100;
const MAX_LOG_SCAN_WINDOW: usize = 10_000;

#[derive(Debug, Deserialize, Default)]
pub struct ListSystemLogsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub date: Option<String>,
    pub level: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SystemLogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SystemLogPage {
    pub list: Vec<SystemLogEntry>,
    pub page: u64,
    pub total: u64,
    pub selected_date: Option<String>,
    pub available_dates: Vec<String>,
}

#[derive(Debug)]
struct SystemLogFilters {
    level: Option<String>,
    keyword: Option<String>,
}

#[handler]
pub async fn list_system_logs(req: &mut Request) -> Result<ApiResponse<SystemLogPage>, AppError> {
    let params = req.parse_queries::<ListSystemLogsParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let page_start_u64 = (page - 1)
        .checked_mul(page_size)
        .ok_or_else(|| AppError::validation("日志分页范围过大"))?;
    let page_end_u64 = page
        .checked_mul(page_size)
        .ok_or_else(|| AppError::validation("日志分页范围过大"))?;
    let page_start =
        usize::try_from(page_start_u64).map_err(|_| AppError::validation("日志分页范围过大"))?;
    let page_end =
        usize::try_from(page_end_u64).map_err(|_| AppError::validation("日志分页范围过大"))?;
    if page_end > MAX_LOG_SCAN_WINDOW {
        return Err(AppError::validation(format!(
            "最多只能查看最近 {MAX_LOG_SCAN_WINDOW} 条匹配日志，请增加筛选条件"
        )));
    }

    let requested_date = normalize_date(params.date)?;
    let filters = SystemLogFilters {
        level: normalize_level(params.level)?,
        keyword: normalize_keyword(params.keyword)?,
    };
    let result = tokio::task::spawn_blocking(move || {
        read_system_log_page(
            Path::new(LOG_DIRECTORY),
            requested_date,
            filters,
            page,
            page_start,
            page_end,
        )
    })
    .await
    .map_err(|error| {
        tracing::error!("System log reader task failed: {error}");
        AppError::InternalError {
            message: "读取系统日志失败".to_string(),
        }
    })?
    .map_err(|error| {
        tracing::error!("Failed to read system logs: {error}");
        AppError::InternalError {
            message: "读取系统日志失败".to_string(),
        }
    })?;

    Ok(ApiResponse::success(result))
}

fn normalize_date(value: Option<String>) -> Result<Option<String>, AppError> {
    let Some(value) = value.map(|item| item.trim().to_string()) else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    NaiveDate::parse_from_str(&value, "%Y-%m-%d")
        .map_err(|_| AppError::validation("日志日期格式必须为 YYYY-MM-DD"))?;
    Ok(Some(value))
}

fn normalize_level(value: Option<String>) -> Result<Option<String>, AppError> {
    let Some(value) = value.map(|item| item.trim().to_ascii_uppercase()) else {
        return Ok(None);
    };
    match value.as_str() {
        "" | "ALL" => Ok(None),
        "INFO" | "WARN" | "ERROR" => Ok(Some(value)),
        "WARNING" => Ok(Some("WARN".to_string())),
        _ => Err(AppError::validation("日志级别只支持 INFO、WARN 或 ERROR")),
    }
}

fn normalize_keyword(value: Option<String>) -> Result<Option<String>, AppError> {
    let Some(value) = value.map(|item| item.trim().to_string()) else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > MAX_KEYWORD_CHARS {
        return Err(AppError::validation(format!(
            "日志关键词不能超过 {MAX_KEYWORD_CHARS} 个字符"
        )));
    }
    Ok(Some(value.to_lowercase()))
}

fn read_system_log_page(
    log_directory: &Path,
    requested_date: Option<String>,
    filters: SystemLogFilters,
    page: u64,
    page_start: usize,
    page_end: usize,
) -> io::Result<SystemLogPage> {
    let available_dates = available_log_dates(log_directory)?;
    let selected_date = requested_date.or_else(|| available_dates.first().cloned());
    let Some(date) = selected_date.as_deref() else {
        return Ok(SystemLogPage {
            list: Vec::new(),
            page,
            total: 0,
            selected_date: None,
            available_dates,
        });
    };

    let file_path = log_directory.join(format!("{LOG_FILE_PREFIX}.{date}"));
    let (list, total) = match File::open(file_path) {
        Ok(file) => scan_log_entries(BufReader::new(file), &filters, page_start, page_end)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => (Vec::new(), 0),
        Err(error) => return Err(error),
    };

    Ok(SystemLogPage {
        list,
        page,
        total: total.min(MAX_LOG_SCAN_WINDOW as u64),
        selected_date: Some(date.to_string()),
        available_dates,
    })
}

fn available_log_dates(log_directory: &Path) -> io::Result<Vec<String>> {
    let entries = match fs::read_dir(log_directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let prefix = format!("{LOG_FILE_PREFIX}.");
    let mut dates = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(date) = name.strip_prefix(&prefix) else {
            continue;
        };
        if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok() {
            dates.push(date.to_string());
        }
    }
    dates.sort_unstable_by(|left, right| right.cmp(left));
    dates.dedup();
    Ok(dates)
}

fn scan_log_entries<R: BufRead>(
    mut reader: R,
    filters: &SystemLogFilters,
    page_start: usize,
    page_end: usize,
) -> io::Result<(Vec<SystemLogEntry>, u64)> {
    let mut recent = VecDeque::with_capacity(page_end.min(1024));
    let mut total = 0_u64;
    let mut current: Option<SystemLogEntry> = None;
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if let Some(entry) = parse_log_line(line) {
            if let Some(previous) = current.replace(entry) {
                retain_matching_entry(previous, filters, page_end, &mut total, &mut recent);
            }
        } else if let Some(entry) = current.as_mut() {
            entry.message.push('\n');
            entry.message.push_str(line);
        }
    }
    if let Some(entry) = current {
        retain_matching_entry(entry, filters, page_end, &mut total, &mut recent);
    }

    let list = recent
        .into_iter()
        .rev()
        .skip(page_start)
        .take(page_end.saturating_sub(page_start))
        .collect();
    Ok((list, total))
}

fn retain_matching_entry(
    entry: SystemLogEntry,
    filters: &SystemLogFilters,
    retain_count: usize,
    total: &mut u64,
    recent: &mut VecDeque<SystemLogEntry>,
) {
    if !entry_matches(&entry, filters) {
        return;
    }
    *total = total.saturating_add(1);
    if retain_count == 0 {
        return;
    }
    recent.push_back(entry);
    if recent.len() > retain_count {
        recent.pop_front();
    }
}

fn entry_matches(entry: &SystemLogEntry, filters: &SystemLogFilters) -> bool {
    if !matches!(entry.level.as_str(), "INFO" | "WARN" | "ERROR") {
        return false;
    }
    if filters
        .level
        .as_ref()
        .is_some_and(|level| level != &entry.level)
    {
        return false;
    }
    let Some(keyword) = filters.keyword.as_deref() else {
        return true;
    };
    entry.timestamp.to_lowercase().contains(keyword)
        || entry.target.to_lowercase().contains(keyword)
        || entry.message.to_lowercase().contains(keyword)
}

fn parse_log_line(line: &str) -> Option<SystemLogEntry> {
    let timestamp = line.get(..23)?;
    NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S%.3f").ok()?;
    let remainder = line.get(23..)?.trim_start();
    let level_end = remainder.find(char::is_whitespace)?;
    let level = &remainder[..level_end];
    if !matches!(level, "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR") {
        return None;
    }
    let remainder = remainder[level_end..].trim_start();
    let (target, message) = remainder
        .split_once(": ")
        .map_or((remainder, ""), |parts| parts);
    Some(SystemLogEntry {
        timestamp: timestamp.to_string(),
        level: level.to_string(),
        target: target.to_string(),
        message: message.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_formatted_tracing_line() {
        let entry = parse_log_line(
            "2026-09-01 12:34:56.789  WARN app_server::payment: notification failed",
        )
        .unwrap();
        assert_eq!(entry.timestamp, "2026-09-01 12:34:56.789");
        assert_eq!(entry.level, "WARN");
        assert_eq!(entry.target, "app_server::payment");
        assert_eq!(entry.message, "notification failed");
    }

    #[test]
    fn filters_and_paginates_newest_entries_first() {
        let source = concat!(
            "2026-09-01 10:00:00.000  INFO app_server::one: first\n",
            "2026-09-01 10:00:30.000 DEBUG app_server::debug: ignored\n",
            "2026-09-01 10:01:00.000 ERROR app_server::two: second failure\n",
            "continued detail\n",
            "2026-09-01 10:02:00.000  INFO app_server::three: third\n",
        );
        let filters = SystemLogFilters {
            level: None,
            keyword: None,
        };
        let (entries, total) = scan_log_entries(Cursor::new(source), &filters, 0, 2).unwrap();
        assert_eq!(total, 3);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "third");
        assert_eq!(entries[1].message, "second failure\ncontinued detail");

        let error_filters = SystemLogFilters {
            level: Some("ERROR".to_string()),
            keyword: Some("detail".to_string()),
        };
        let (entries, total) =
            scan_log_entries(Cursor::new(source), &error_filters, 0, 20).unwrap();
        assert_eq!(total, 1);
        assert_eq!(entries[0].target, "app_server::two");
    }

    #[test]
    fn validates_filter_values() {
        assert_eq!(
            normalize_level(Some("warning".to_string())).unwrap(),
            Some("WARN".to_string())
        );
        assert!(normalize_level(Some("fatal".to_string())).is_err());
        assert!(normalize_date(Some("../../secret".to_string())).is_err());
    }
}
