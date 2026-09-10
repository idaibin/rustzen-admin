use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const MONITOR_ROUTES: &str = "403.tsx\n404.tsx\n__root.tsx\nindex.tsx\nlogin.tsx\nmonitoring.tsx\nmonitoring/-global-alert-settings.tsx\nmonitoring/-incident-drawer.tsx\nmonitoring/-node-alert-policy.tsx\nmonitoring/-node-details.tsx\nmonitoring/-node-onboarding.tsx\nmonitoring/-save-state.ts\nmonitoring/incidents.tsx\nmonitoring/nodes.tsx\nmonitoring/overview.tsx\nmonitoring/summaries.tsx\nprofile.tsx\nsystem/-role-actions.tsx\nsystem/-role-delete-state.ts\nsystem/-role-dialog.tsx\nsystem/-role-permission-picker.tsx\nsystem/-user-actions.tsx\nsystem/-user-dialog.tsx\nsystem/role.tsx\nsystem/user.tsx";
const MONITOR_NOTIFY_ROUTES: &str = "-notifications-shell.tsx\n403.tsx\n404.tsx\n__root.tsx\nindex.tsx\nlogin.tsx\nmonitoring.tsx\nmonitoring/-global-alert-settings.tsx\nmonitoring/-incident-drawer.tsx\nmonitoring/-node-alert-policy.tsx\nmonitoring/-node-details.tsx\nmonitoring/-node-onboarding.tsx\nmonitoring/-save-state.ts\nmonitoring/incidents.tsx\nmonitoring/nodes.tsx\nmonitoring/overview.tsx\nmonitoring/summaries.tsx\nprofile.tsx\nsystem/-role-actions.tsx\nsystem/-role-delete-state.ts\nsystem/-role-dialog.tsx\nsystem/-role-permission-picker.tsx\nsystem/-user-actions.tsx\nsystem/-user-dialog.tsx\nsystem/role.tsx\nsystem/user.tsx";

struct SelectedWeb {
    preset: &'static str,
    composition_id: &'static str,
    root: &'static str,
    generated_root: &'static str,
    output_directory: &'static str,
}

pub fn selected_web_root() -> &'static str {
    selected_web().root
}

fn selected_web() -> SelectedWeb {
    if std::env::var_os("CARGO_FEATURE_NOTIFICATIONS").is_some() {
        SelectedWeb {
            preset: "monitor-notify",
            composition_id: "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d",
            root: "selected-web/0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d",
            generated_root: "apps/web/.selected-web/0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d",
            output_directory: "target/distributions/0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d/web/dist",
        }
    } else {
        SelectedWeb {
            preset: "monitor",
            composition_id: "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b",
            root: "selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b",
            generated_root: "apps/web/.selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b",
            output_directory: "target/distributions/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b/web/dist",
        }
    }
}

pub fn validate_selected_web() {
    let selected = selected_web();
    let binding_path = format!("{}/binding.json", selected.root);
    let inventory_path = format!("{}/inventory.json", selected.root);
    let api_path = format!("{}/api.ts", selected.root);
    let dist_path = format!("{}/dist", selected.root);
    let index_path = format!("{}/dist/index.html", selected.root);
    let binding = read_json(&binding_path, "binding");
    let inventory = read_json(&inventory_path, "inventory");
    let binding_object = binding.as_object().expect("selected Web binding must be an object");
    let mut keys = binding_object.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["bindingVersion", "compositionId", "selectedApiDigest", "webDigest"],
        "selected Web binding schema mismatch"
    );
    assert_eq!(binding["bindingVersion"], 1, "selected Web binding version mismatch");
    assert_eq!(
        binding["compositionId"].as_str(),
        Some(selected.composition_id),
        "selected Web binding composition mismatch"
    );
    for field in ["compositionId", "selectedApiDigest", "webDigest"] {
        assert!(
            binding[field].as_str().is_some_and(is_sha256),
            "selected Web binding {field} must be a lowercase SHA-256"
        );
    }
    validate_inventory_contract(&inventory, &selected);
    assert_eq!(
        inventory["compositionId"], binding["compositionId"],
        "selected Web inventory composition differs from binding"
    );
    assert_eq!(inventory["binding"], binding, "selected Web inventory binding differs");

    let selected_api = std::fs::read(&api_path)
        .unwrap_or_else(|_| panic!("monitor distribution requires selected Web API: {api_path}"));
    assert_eq!(
        binding["selectedApiDigest"].as_str(),
        Some(sha256(&selected_api).as_str()),
        "selected Web API digest mismatch"
    );

    let index = std::fs::read_to_string(&index_path).unwrap_or_else(|_| {
        panic!("monitor distribution requires selected Web HTML: {index_path}")
    });
    let digest = binding["webDigest"].as_str().expect("validated Web digest");
    let expected = format!(r#"<meta name="rustzen-web-binding" content="{digest}" />"#);
    assert_eq!(
        index.match_indices("rustzen-web-binding").count(),
        1,
        "selected Web HTML must contain one binding marker"
    );
    assert_eq!(
        index.match_indices(&expected).count(),
        1,
        "selected Web HTML binding stamp mismatch"
    );
    assert!(
        !index.contains("__RUSTZEN_WEB_DIGEST__"),
        "selected Web HTML still contains the binding slot"
    );
    validate_dist(&inventory, Path::new(&dist_path), digest);
    println!("cargo:rustc-env=RUSTZEN_PACKAGED_WEB_DIGEST={digest}");
    println!("cargo:rustc-env=RUSTZEN_PACKAGED_WEB_COMPOSITION_ID={}", selected.composition_id);
}

fn validate_inventory_contract(inventory: &Value, selected: &SelectedWeb) {
    let object = inventory.as_object().expect("selected Web inventory must be an object");
    let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    let mut expected = [
        "schemaVersion",
        "preset",
        "compositionId",
        "generatedRoot",
        "outputDirectory",
        "selectedRoutes",
        "publicAssets",
        "emittedFiles",
        "fileInventory",
        "moduleIds",
        "binding",
    ];
    expected.sort_unstable();
    assert_eq!(keys, expected, "selected Web inventory schema mismatch");
    assert_eq!(inventory["schemaVersion"], 2, "selected Web inventory version mismatch");
    assert_eq!(inventory["preset"], selected.preset, "selected Web inventory preset mismatch");
    assert_eq!(
        inventory["compositionId"].as_str(),
        Some(selected.composition_id),
        "selected Web inventory composition mismatch"
    );
    assert_eq!(
        inventory["generatedRoot"], selected.generated_root,
        "selected Web generated root mismatch"
    );
    assert_eq!(
        inventory["outputDirectory"], selected.output_directory,
        "selected Web output mismatch"
    );
    let expected_routes =
        if selected.preset == "monitor" { MONITOR_ROUTES } else { MONITOR_NOTIFY_ROUTES };
    assert_eq!(
        string_array(&inventory["selectedRoutes"], "selected routes"),
        expected_routes.split('\n').collect::<Vec<_>>(),
        "selected Web inventory route set mismatch"
    );
    assert_eq!(
        string_array(&inventory["publicAssets"], "public assets"),
        ["rustzen.png"],
        "selected Web inventory public assets mismatch"
    );
    let modules = string_array(&inventory["moduleIds"], "module IDs");
    let prefix = format!("apps/web/.selected-web/{}/", selected.composition_id);
    crate::module_policy::validate(&modules, &prefix, selected.preset == "monitor-notify");
}

fn string_array<'a>(value: &'a Value, label: &str) -> Vec<&'a str> {
    let values =
        value.as_array().unwrap_or_else(|| panic!("selected Web {label} must be an array"));
    let result = values
        .iter()
        .map(|item| item.as_str().unwrap_or_else(|| panic!("selected Web {label} must be strings")))
        .collect::<Vec<_>>();
    assert!(
        result.windows(2).all(|pair| pair[0] < pair[1]),
        "selected Web {label} must be sorted and unique"
    );
    result
}

fn validate_dist(inventory: &Value, root: &Path, web_digest: &str) {
    let rows = inventory["fileInventory"]
        .as_array()
        .expect("selected Web inventory file table must be an array");
    let mut expected = Vec::with_capacity(rows.len());
    for row in rows {
        let object = row.as_object().expect("selected Web file entry must be an object");
        let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        keys.sort_unstable();
        assert_eq!(keys, ["path", "sha256", "size"], "selected Web file schema mismatch");
        let path = row["path"].as_str().expect("selected Web file path must be a string");
        assert_safe_path(path);
        let size = row["size"].as_u64().expect("selected Web file size must be unsigned");
        let digest = row["sha256"]
            .as_str()
            .filter(|value| is_sha256(value))
            .expect("selected Web file digest must be a lowercase SHA-256");
        let bytes = std::fs::read(root.join(path))
            .unwrap_or_else(|_| panic!("selected Web inventory file is missing: {path}"));
        assert_eq!(bytes.len() as u64, size, "selected Web file size mismatch: {path}");
        assert_eq!(sha256(&bytes), digest, "selected Web file digest mismatch: {path}");
        expected.push(path.to_owned());
    }
    assert!(
        expected.windows(2).all(|pair| pair[0] < pair[1]),
        "selected Web file inventory must be sorted and unique"
    );
    let mut actual = Vec::new();
    collect_files(root, root, &mut actual);
    actual.sort();
    assert_eq!(actual, expected, "selected Web inventory file set mismatch");
    assert_eq!(
        string_array(&inventory["emittedFiles"], "emitted files"),
        actual,
        "selected Web emitted files mismatch"
    );

    let mut normalized = expected
        .iter()
        .map(|path| {
            let mut bytes = std::fs::read(root.join(path)).expect("validated selected Web file");
            if path == "index.html" {
                let stamped =
                    format!(r#"<meta name="rustzen-web-binding" content="{web_digest}" />"#);
                let text = String::from_utf8(bytes).expect("selected Web HTML must be UTF-8");
                bytes = text
                    .replace(
                        &stamped,
                        r#"<meta name="rustzen-web-binding" content="__RUSTZEN_WEB_DIGEST__" />"#,
                    )
                    .into_bytes();
            }
            serde_json::json!({"path": path, "sha256": sha256(&bytes)})
        })
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    let canonical = serde_json::to_vec(&normalized).expect("selected Web digest table serializes");
    assert_eq!(sha256(&canonical), web_digest, "selected Web normalized digest mismatch");
}

fn collect_files(directory: &Path, root: &Path, files: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).expect("selected Web directory must be readable") {
        let entry = entry.expect("selected Web directory entry must be readable");
        let kind = entry.file_type().expect("selected Web file type must be readable");
        assert!(!kind.is_symlink(), "selected Web output must not contain symlinks");
        if kind.is_dir() {
            collect_files(&entry.path(), root, files);
        } else {
            assert!(kind.is_file(), "selected Web output contains unsupported entry");
            files.push(relative_path(entry.path(), root));
        }
    }
}

fn relative_path(path: PathBuf, root: &Path) -> String {
    path.strip_prefix(root)
        .expect("selected Web file must remain below root")
        .components()
        .map(|part| part.as_os_str().to_str().expect("selected Web path must be UTF-8"))
        .collect::<Vec<_>>()
        .join("/")
}

fn assert_safe_path(path: &str) {
    assert!(
        !path.is_empty()
            && !path.starts_with('/')
            && !path.contains('\\')
            && path.split('/').all(|part| !part.is_empty() && part != "." && part != ".."),
        "selected Web inventory path is unsafe"
    );
}

fn read_json(path: &str, label: &str) -> Value {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|_| panic!("monitor distribution requires selected Web {label}: {path}"));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| panic!("selected Web {label} must be valid JSON: {path}"))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
