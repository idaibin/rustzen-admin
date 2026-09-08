use crate::{
    install::Manifest,
    install_crypto::{canonical_json, hash},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SLOT: &str = "__RUSTZEN_WEB_DIGEST__";
const MARKER: &str = "rustzen-web-binding";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Binding {
    binding_version: u8,
    composition_id: String,
    selected_api_digest: String,
    web_digest: String,
}

pub(super) fn validate(manifest: &Manifest, files: &BTreeMap<String, &[u8]>) -> Result<(), String> {
    if manifest.artifact_class != "server" {
        return Ok(());
    }
    let bytes = files.get("contracts/web/binding.json").ok_or("selected Web binding is missing")?;
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| "selected Web binding is invalid JSON")?;
    if canonical_json(&value)? != *bytes {
        return Err("selected Web binding bytes are not canonical".into());
    }
    let binding: Binding =
        serde_json::from_value(value).map_err(|_| "selected Web binding fields are invalid")?;
    let manifest_digest = &manifest.web_digest.as_ref().ok_or("server web digest missing")?.sha256;
    if binding.binding_version != 1
        || !super::hash_id(&binding.composition_id)
        || !super::hash_id(&binding.selected_api_digest)
        || !super::hash_id(&binding.web_digest)
        || binding.composition_id != manifest.composition_id
        || &binding.web_digest != manifest_digest
    {
        return Err("selected Web binding differs from manifest".into());
    }
    let mut entries = Vec::new();
    for (path, bytes) in files.iter().filter(|(path, _)| path.starts_with("web/")) {
        let relative = path.strip_prefix("web/").ok_or("selected Web path is invalid")?;
        if !super::path(relative) {
            return Err("selected Web path is invalid".into());
        }
        let digest = if relative == "index.html" {
            hash(&normalize_index(bytes, &binding.web_digest)?)
        } else {
            hash(bytes)
        };
        entries.push(json!({"path": relative, "sha256": digest}));
    }
    if entries.is_empty() || !files.contains_key("web/index.html") {
        return Err("selected Web payload is incomplete".into());
    }
    if hash(&canonical_json(&Value::Array(entries))?) != binding.web_digest {
        return Err("server web digest differs from payload".into());
    }
    Ok(())
}

fn normalize_index(bytes: &[u8], digest: &str) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "selected Web index is not UTF-8")?;
    if text.to_ascii_lowercase().match_indices(MARKER).count() != 1 {
        return Err("selected Web index binding marker is ambiguous".into());
    }
    let stamp = format!(r#"<meta name="rustzen-web-binding" content="{digest}" />"#);
    if text.match_indices(&stamp).count() != 1 {
        return Err("selected Web index binding stamp differs".into());
    }
    Ok(text
        .replacen(&stamp, &format!(r#"<meta name="rustzen-web-binding" content="{SLOT}" />"#), 1)
        .into_bytes())
}
