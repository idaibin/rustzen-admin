#[path = "build_support/module_policy.rs"]
mod module_policy;
#[path = "build_support/selected_web.rs"]
mod selected_web;

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed={}", selected_web::selected_web_root());
    if std::env::var_os("CARGO_FEATURE_SELECTED_DISTRIBUTION").is_some() {
        selected_web::validate_selected_web();
    }
}
