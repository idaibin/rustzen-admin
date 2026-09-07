mod app;
mod common;
mod config;
mod features;
mod infra;
mod middleware;
#[cfg(feature = "notifications")]
mod notifications;

use crate::{config::CONFIG, infra::logger::init_logging};

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_RELEASE_MARKER: &str = concat!(
    "RUSTZEN_RELEASE_MARKER\n",
    "artifact=rz-bundle-member\n",
    "binary=rz-reports\n",
    "version=",
    env!("CARGO_PKG_VERSION"),
    "\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().skip(1).collect::<Vec<_>>() == ["contract", "config", "selected"] {
        println!("{}", serde_json::to_string(&rustzen_config::reports_contract())?);
        return Ok(());
    }
    rustzen_config::load_dotenv_if_present()?;
    let command = Command::parse(std::env::args().skip(1))?;
    // SAFETY: this runs in synchronous main before Tokio creates worker threads.
    unsafe { rustzen_config::initialize_process_timezone(CONFIG.timezone()) };
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        let _logging = init_logging()?;
        match command {
            Command::Serve => app::run_server().await,
        }
    })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Command {
    Serve,
}

impl Command {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, CommandError> {
        match args.into_iter().collect::<Vec<_>>().as_slice() {
            [mode] if mode == "serve" => Ok(Self::Serve),
            _ => Err(CommandError),
        }
    }
}

#[derive(Debug)]
struct CommandError;

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("usage: rz-reports serve")
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::{Command, CommandError};

    #[test]
    fn accepts_only_the_serve_mode() {
        assert_eq!(Command::parse(["serve".to_string()]).ok(), Some(Command::Serve));
        assert!(matches!(Command::parse(std::iter::empty()), Err(CommandError)));
        assert!(Command::parse(["worker".to_string()]).is_err());
    }

    #[test]
    fn local_reports_startup_configuration_is_valid() {
        rustzen_config::ReportsConfig::local().expect("local Reports startup config");
    }
}
