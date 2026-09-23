#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Command {
    Serve,
    UpdateWorker(i64),
    UpdateRequestWorker,
    UpdateRecover,
    OpenApi,
}

impl Command {
    pub(crate) fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, CommandError> {
        match args.into_iter().collect::<Vec<_>>().as_slice() {
            [mode] if mode == "serve" => Ok(Self::Serve),
            [mode] if mode == "openapi" => Ok(Self::OpenApi),
            [module, mode] if module == "update" && mode == "recover" => Ok(Self::UpdateRecover),
            [module, mode] if module == "update" && mode == "request-worker" => {
                Ok(Self::UpdateRequestWorker)
            }
            [module, mode, id] if module == "update" && mode == "worker" => id
                .parse::<i64>()
                .ok()
                .filter(|id| *id > 0)
                .map(Self::UpdateWorker)
                .ok_or(CommandError),
            _ => Err(CommandError),
        }
    }
}

#[derive(Debug)]
pub(crate) struct CommandError;

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("usage: rz-admin serve | rz-admin openapi | rz-admin update worker <release-id> | rz-admin update request-worker | rz-admin update recover")
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn parses_admin_commands_only() {
        assert_eq!(Command::parse(["serve".to_string()]).ok(), Some(Command::Serve));
        assert_eq!(Command::parse(["openapi".to_string()]).ok(), Some(Command::OpenApi));
        assert_eq!(
            Command::parse(["update".to_string(), "worker".to_string(), "7".to_string()]).ok(),
            Some(Command::UpdateWorker(7))
        );
        assert_eq!(
            Command::parse(["update".to_string(), "request-worker".to_string()]).ok(),
            Some(Command::UpdateRequestWorker)
        );
        assert_eq!(
            Command::parse(["update".to_string(), "recover".to_string()]).ok(),
            Some(Command::UpdateRecover)
        );
        assert!(Command::parse(["monitor".to_string(), "controller".to_string()]).is_err());
        assert!(Command::parse(std::iter::empty()).is_err());
    }
}
