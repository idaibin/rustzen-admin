#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Command {
    Serve,
    #[cfg(feature = "selected-distribution")]
    BootstrapOwner,
    #[cfg(feature = "selected-distribution")]
    VerifyOwner,
    #[cfg(feature = "selected-distribution")]
    ValidateConfig,
    #[cfg(feature = "selected-distribution")]
    ValidateDatabase,
    #[cfg(feature = "selected-distribution")]
    BindDatabase,
    #[cfg(feature = "selected-distribution")]
    ContractSelected(String),
    #[cfg(feature = "selected-distribution")]
    ContractProtocol,
    #[cfg(feature = "selected-distribution")]
    ContractConfigSelected(String),
    #[cfg(feature = "full")]
    UpdateWorker(i64),
    #[cfg(feature = "full")]
    UpdateRecover,
    #[cfg(feature = "full")]
    OpenApi,
}

impl Command {
    pub(crate) fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, CommandError> {
        let args = args.into_iter().collect::<Vec<_>>();
        match args.as_slice() {
            [mode] if mode == "serve" => Ok(Self::Serve),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "bootstrap-owner" => Ok(Self::BootstrapOwner),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "verify-owner" => Ok(Self::VerifyOwner),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "validate-config" => Ok(Self::ValidateConfig),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "validate-database" => Ok(Self::ValidateDatabase),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "bind-database" => Ok(Self::BindDatabase),
            #[cfg(feature = "selected-distribution")]
            [domain, mode, owner]
                if domain == "contract"
                    && mode == "selected"
                    && (owner == "admin" || owner == "notifications") =>
            {
                Ok(Self::ContractSelected(owner.clone()))
            }
            #[cfg(feature = "selected-distribution")]
            [domain, mode] if domain == "contract" && mode == "protocol" => {
                Ok(Self::ContractProtocol)
            }
            #[cfg(feature = "selected-distribution")]
            [domain, kind, mode, owner]
                if domain == "contract"
                    && kind == "config"
                    && mode == "selected"
                    && selected_config_owner(owner) =>
            {
                Ok(Self::ContractConfigSelected(owner.clone()))
            }
            #[cfg(feature = "full")]
            [mode] if mode == "openapi" => Ok(Self::OpenApi),
            #[cfg(feature = "full")]
            [module, mode] if module == "update" && mode == "recover" => Ok(Self::UpdateRecover),
            #[cfg(feature = "full")]
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

#[cfg(feature = "selected-distribution")]
fn selected_config_owner(owner: &str) -> bool {
    owner == "access" || (cfg!(feature = "notifications") && owner == "notifications")
}

#[derive(Debug)]
pub(crate) struct CommandError;

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "full")]
        let usage = "usage: rz-admin serve | rz-admin openapi | rz-admin update worker <release-id> | rz-admin update recover";
        #[cfg(not(feature = "full"))]
        let usage = "usage: rz-admin serve | rz-admin bootstrap-owner | rz-admin verify-owner | rz-admin validate-config | rz-admin bind-database | rz-admin validate-database | rz-admin contract selected <admin|notifications> | rz-admin contract protocol | rz-admin contract config selected <access|notifications>";
        formatter.write_str(usage)
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn parses_admin_commands_only() {
        assert_eq!(Command::parse(["serve".to_string()]).ok(), Some(Command::Serve));
        #[cfg(feature = "full")]
        assert_eq!(Command::parse(["openapi".to_string()]).ok(), Some(Command::OpenApi));
        #[cfg(feature = "full")]
        assert_eq!(
            Command::parse(["update".to_string(), "worker".to_string(), "7".to_string()]).ok(),
            Some(Command::UpdateWorker(7))
        );
        #[cfg(feature = "full")]
        assert_eq!(
            Command::parse(["update".to_string(), "recover".to_string()]).ok(),
            Some(Command::UpdateRecover)
        );
        assert!(Command::parse(["monitor".to_string(), "controller".to_string()]).is_err());
        #[cfg(feature = "selected-distribution")]
        assert_eq!(
            Command::parse(["contract".to_string(), "selected".to_string(), "admin".to_string(),])
                .ok(),
            Some(Command::ContractSelected("admin".into()))
        );
        #[cfg(feature = "selected-distribution")]
        assert_eq!(
            Command::parse(["contract".to_string(), "protocol".to_string()]).ok(),
            Some(Command::ContractProtocol)
        );
        #[cfg(feature = "selected-distribution")]
        assert_eq!(
            Command::parse(["validate-database".to_string()]).ok(),
            Some(Command::ValidateDatabase)
        );
        #[cfg(feature = "selected-distribution")]
        assert_eq!(Command::parse(["bind-database".to_string()]).ok(), Some(Command::BindDatabase));
        #[cfg(feature = "selected-distribution")]
        assert_eq!(
            Command::parse([
                "contract".to_string(),
                "config".to_string(),
                "selected".to_string(),
                "access".to_string(),
            ])
            .ok(),
            Some(Command::ContractConfigSelected("access".into()))
        );
        #[cfg(all(feature = "selected-distribution", feature = "notifications"))]
        assert_eq!(
            Command::parse([
                "contract".to_string(),
                "config".to_string(),
                "selected".to_string(),
                "notifications".to_string(),
            ])
            .ok(),
            Some(Command::ContractConfigSelected("notifications".into()))
        );
        #[cfg(all(feature = "selected-distribution", not(feature = "notifications")))]
        assert!(
            Command::parse([
                "contract".to_string(),
                "config".to_string(),
                "selected".to_string(),
                "notifications".to_string(),
            ])
            .is_err()
        );
        #[cfg(feature = "selected-distribution")]
        assert!(Command::parse(["openapi".to_string()]).is_err());
        assert!(Command::parse(std::iter::empty()).is_err());
    }

    #[test]
    fn local_admin_startup_configuration_is_valid() {
        rustzen_config::AdminConfig::local().expect("local Admin startup config");
    }
}
