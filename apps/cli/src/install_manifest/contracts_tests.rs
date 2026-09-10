use super::*;
use crate::install::{BinaryDigest, DigestRecord, Entry, Manifest};

fn manifest(preset: &str) -> Manifest {
    Manifest {
        manifest_version: 1,
        release_class: "production".into(),
        release_version: "test".into(),
        target: "x86_64-unknown-linux-musl".into(),
        artifact_class: "server".into(),
        preset: preset.into(),
        capabilities: vec![],
        services: vec![],
        composition_id: "a".repeat(64),
        selection_digest: DigestRecord {
            sha256: "a".repeat(64),
            source: "resolved-selection".into(),
        },
        build_id: "a".repeat(64),
        source_identity: "test".into(),
        config_digest: "a".repeat(64),
        native_layout_digest: "a".repeat(64),
        protocol_artifact_digest: "a".repeat(64),
        config_owners: vec![],
        binary_digests: Vec::<BinaryDigest>::new(),
        agent_protocol_contract_id: "a".repeat(64),
        files: Vec::<Entry>::new(),
        api_digest: None,
        schema_fingerprints: None,
        data_contract_ids: None,
        web_digest: None,
    }
}
fn schema(owners: &[&str], preset: &str) -> serde_json::Map<String, Value> {
    serde_json::json!({"preset":preset,"compositionId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","owners":owners.iter().map(|owner| (owner.to_string(), serde_json::json!({"dataContractId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","schemaSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}))).collect::<serde_json::Map<String, Value>>() }).as_object().unwrap().clone()
}
#[test]
fn schema_owner_sets_are_preset_exact() {
    assert!(
        schema_identity(&schema(&["admin", "monitor"], "monitor"), &manifest("monitor")).is_ok()
    );
    assert!(
        schema_identity(
            &schema(
                &["admin", "admin-notifications", "monitor", "monitor-notifications"],
                "monitor-notify"
            ),
            &manifest("monitor-notify")
        )
        .is_ok()
    );
    assert!(
        schema_identity(
            &schema(&["admin", "monitor"], "monitor-notify"),
            &manifest("monitor-notify")
        )
        .is_err()
    );
    assert!(
        schema_identity(
            &schema(
                &["admin", "admin-notifications", "monitor", "monitor-notifications", "extra"],
                "monitor-notify"
            ),
            &manifest("monitor-notify")
        )
        .is_err()
    );
}
