use serde::Deserialize;

/// Log query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQueryDto {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
    pub ip_address: Option<String>,
}
