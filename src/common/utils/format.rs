/// 格式化工具函数集合
pub struct FormatUtils;

const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
impl FormatUtils {
    /// 格式化文件大小
    pub fn format_file_size(bytes: u64) -> String {
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// 格式化数字（添加千分位分隔符）
    pub fn format_number(num: i64) -> String {
        let num_str = num.to_string();
        let mut result = String::new();
        let chars: Vec<char> = num_str.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(ch);
        }
        result
    }
}
