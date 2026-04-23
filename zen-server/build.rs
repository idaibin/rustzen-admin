fn main() {
    // Rebuild when embedded migration files change so sqlx::migrate! stays in sync.
    println!("cargo:rerun-if-changed=migrations");
}
