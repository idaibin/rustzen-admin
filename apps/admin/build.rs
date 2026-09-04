const COMPOSITION_ID: &str = "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const SELECTED_WEB_ROOT: &str =
    "selected-web/8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed={SELECTED_WEB_ROOT}");
    if std::env::var_os("CARGO_FEATURE_MONITOR_DISTRIBUTION").is_some() {
        let inventory = format!("{SELECTED_WEB_ROOT}/inventory.json");
        let contents = std::fs::read_to_string(&inventory).unwrap_or_else(|_| {
            panic!("monitor distribution requires verified selected Web inventory: {inventory}")
        });
        assert!(
            contents.contains(&format!(r#""compositionId": "{COMPOSITION_ID}""#)),
            "selected Web inventory composition mismatch"
        );
        assert!(
            std::path::Path::new(&format!("{SELECTED_WEB_ROOT}/dist/index.html")).is_file(),
            "selected Web artifact is missing index.html"
        );
    }
}
