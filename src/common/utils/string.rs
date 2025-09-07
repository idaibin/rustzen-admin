/// 字符串工具函数集合
pub struct StringUtils;

impl StringUtils {
    /// 隐藏邮箱中间部分
    pub fn mask_email(email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let (username, domain) = email.split_at(at_pos);
            if username.len() <= 2 {
                return email.to_string();
            }
            let masked_username =
                format!("{}***{}", &username[..1], &username[username.len() - 1..]);
            format!("{}@{}", masked_username, &domain[1..])
        } else {
            email.to_string()
        }
    }

    /// 隐藏手机号中间部分
    pub fn mask_phone(phone: &str) -> String {
        if phone.len() >= 7 {
            format!("{}****{}", &phone[..3], &phone[phone.len() - 4..])
        } else {
            phone.to_string()
        }
    }

    /// 截断字符串并添加省略号
    pub fn truncate(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len.saturating_sub(3)])
        }
    }
}
