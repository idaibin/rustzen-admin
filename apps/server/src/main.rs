mod common;
mod features;
mod infra;
mod middleware;

use crate::infra::app::run_server;
use crate::infra::config::CONFIG;
use crate::infra::logger::init_logging;

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_ADMIN_MARKER: &str = concat!(
    "RUSTZEN_ADMIN_MARKER\n",
    "component=server\n",
    "build_id=",
    env!("CARGO_PKG_VERSION"),
    "\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load env
    dotenvy::dotenv().ok();

    init_process_timezone();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        // init log
        let _logging = init_logging()?;

        // create server
        run_server().await
    })?;

    Ok(())
}

fn init_process_timezone() {
    // SAFETY: this runs in sync main before the Tokio runtime and worker threads are created.
    unsafe {
        std::env::set_var("TZ", CONFIG.timezone.trim());
    }
}
