mod app;
mod common;
mod config;
mod features;
mod infra;
mod middleware;
mod module_routes;
pub mod protocol;
mod protocol_contract;
mod selected_contract;

use crate::{app::run_controller, infra::logger::init_logging};

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_RELEASE_MARKER: &str = concat!(
    "RUSTZEN_RELEASE_MARKER\n",
    "artifact=rz-bundle-member\n",
    "binary=rz-monitor\n",
    "version=",
    env!("CARGO_PKG_VERSION"),
    "\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().skip(1).collect::<Vec<_>>() == ["contract", "protocol"] {
        println!("{}", protocol::contract_protocol_output());
        return Ok(());
    }
    if std::env::args().skip(1).collect::<Vec<_>>() == ["contract", "selected"] {
        println!("{}", selected_contract::selected_contract_json()?);
        return Ok(());
    }
    rustzen_config::load_dotenv_if_present()?;
    let command = Command::parse(std::env::args().skip(1))?;
    let _ = command;
    run_controller_process()
}

fn run_controller_process() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::controller();
    // SAFETY: this runs before Tokio creates worker threads.
    unsafe { rustzen_config::initialize_process_timezone(config.timezone()) };
    let log_dir = config.log_dir();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        let _logging = init_logging(log_dir)?;
        run_controller().await
    })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Command {
    Controller,
}

impl Command {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, CommandError> {
        match args.into_iter().collect::<Vec<_>>().as_slice() {
            [mode] if mode == "controller" => Ok(Self::Controller),
            _ => Err(CommandError),
        }
    }
}

#[derive(Debug)]
struct CommandError;

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("usage: rz-monitor controller")
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::{Command, CommandError};

    #[test]
    fn parses_controller_mode_only() {
        assert_eq!(Command::parse(["controller".to_string()]).ok(), Some(Command::Controller));
        assert!(Command::parse(["agent".to_string()]).is_err());
        assert!(matches!(Command::parse(std::iter::empty()), Err(CommandError)));
        assert!(Command::parse(["monitor".to_string(), "controller".to_string()]).is_err());
    }

    #[test]
    fn local_monitor_startup_configurations_are_valid_and_mode_focused() {
        rustzen_config::MonitorControllerConfig::local()
            .expect("local Monitor Controller startup config");
    }
}
