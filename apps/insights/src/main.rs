mod app;
mod common;
mod config;
mod features;
mod infra;
mod middleware;

use std::error::Error;

use crate::{config::CONFIG, infra::logger::init_logging};

type StartupResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_RELEASE_MARKER: &str = concat!(
    "RUSTZEN_RELEASE_MARKER\n",
    "artifact=rz-bundle-member\n",
    "binary=rz-insights\n",
    "version=",
    env!("CARGO_PKG_VERSION"),
    "\n",
);

fn main() -> StartupResult<()> {
    rustzen_config::load_dotenv_if_present()?;
    let command = Command::parse(std::env::args().skip(1))?;
    // SAFETY: this runs in synchronous main before Tokio creates worker threads.
    unsafe { rustzen_config::initialize_process_timezone(CONFIG.timezone()) };
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        let _logging = init_logging().map_err(|error| std::io::Error::other(error.to_string()))?;
        match command {
            Command::Serve => app::run().await,
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
        formatter.write_str("usage: rz-insights serve")
    }
}

impl Error for CommandError {}

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
    fn local_insights_startup_configuration_is_valid() {
        rustzen_config::InsightsConfig::local().expect("local Insights startup config");
    }
}
