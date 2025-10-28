use chrono::{NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// 时间工具函数集合
pub struct TimeUtils;

impl TimeUtils {
    /// 获取当前时间戳（秒）
    pub fn now_timestamp() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    /// 获取当前时间戳（毫秒）
    pub fn now_timestamp_ms() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
    }

    /// 格式化时间显示
    pub fn format_duration(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;

        if days > 0 {
            format!("{}天{}小时", days, hours)
        } else if hours > 0 {
            format!("{}小时{}分钟", hours, minutes)
        } else {
            format!("{}分钟", minutes)
        }
    }

    /// 相对时间显示（如：2小时前）
    pub fn relative_time(datetime: NaiveDateTime) -> String {
        let now = Utc::now().naive_utc();
        let diff = now - datetime;

        if diff.num_days() > 0 {
            format!("{}天前", diff.num_days())
        } else if diff.num_hours() > 0 {
            format!("{}小时前", diff.num_hours())
        } else if diff.num_minutes() > 0 {
            format!("{}分钟前", diff.num_minutes())
        } else {
            "刚刚".to_string()
        }
    }
}
