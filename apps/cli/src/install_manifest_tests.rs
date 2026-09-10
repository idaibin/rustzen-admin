use super::*;

fn manifest(preset: &str, capabilities: &[&str], owners: &[&str]) -> Manifest {
    Manifest {
        manifest_version: 1,
        release_class: "production".into(),
        release_version: "test".into(),
        target: "x86_64-unknown-linux-musl".into(),
        artifact_class: "server".into(),
        preset: preset.into(),
        capabilities: capabilities.iter().map(|x| (*x).into()).collect(),
        services: vec!["admin".into(), "monitor".into()],
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
        config_owners: owners.iter().map(|x| (*x).into()).collect(),
        binary_digests: vec![],
        agent_protocol_contract_id: "a".repeat(64),
        files: vec![],
        api_digest: None,
        schema_fingerprints: None,
        data_contract_ids: None,
        web_digest: None,
    }
}
#[test]
fn server_rejects_cross_preset_capabilities_and_owners() {
    let files = BTreeMap::new();
    assert!(
        server(
            &manifest("monitor", &["access", "monitor", "notifications"], &["access", "monitor"]),
            &files
        )
        .is_err()
    );
    assert!(
        server(
            &manifest(
                "monitor-notify",
                &["access", "monitor"],
                &["access", "monitor", "notifications"]
            ),
            &files
        )
        .is_err()
    );
    assert!(
        server(
            &manifest(
                "monitor-notify",
                &["access", "monitor", "notifications"],
                &["access", "monitor"]
            ),
            &files
        )
        .is_err()
    );
}

#[test]
fn selected_server_contract_accepts_only_exact_preset_tuple() {
    let monitor = vec!["access".into(), "monitor".into()];
    let notify = vec!["access".into(), "monitor".into(), "notifications".into()];
    let monitor_id = crate::install_crypto::hash(b"{\"artifactClass\":\"server\",\"capabilities\":[\"access\",\"monitor\"],\"capabilityContractVersion\":1}");
    assert!(selected_server_selection("monitor", &monitor, &monitor, &monitor_id));
    assert!(selected_server_selection(
        "monitor-notify",
        &notify,
        &notify,
        "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d"
    ));
    assert!(!selected_server_selection("monitor", &notify, &notify, &monitor_id));
    assert!(!selected_server_selection(
        "monitor-notify",
        &notify,
        &monitor,
        "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d"
    ));
    assert!(!selected_server_selection("monitor-notify", &notify, &notify, &monitor_id));
}
