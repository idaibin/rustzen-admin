fn main() {
    println!("cargo:rerun-if-changed=../../apps/server/migrations/sqlite");
}
