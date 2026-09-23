use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebBindingResponse {
    pub binding_version: u8,
    pub web_digest: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationResponse {
    pub release_version: &'static str,
    pub composition_id: String,
    pub build_id: String,
    pub web_digest: &'static str,
    pub feature_ids: Vec<&'static str>,
    pub services: Vec<SelectedServiceResponse>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectedServiceResponse {
    pub id: String,
    pub state: SelectedServiceState,
    pub release_version: Option<String>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SelectedServiceState {
    Healthy,
    Unavailable,
    Incompatible,
}
