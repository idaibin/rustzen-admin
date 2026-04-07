use super::super::auth::CurrentUser;

#[derive(Debug, Clone)]
pub enum PermissionsCheck {
    Require(&'static str),
    Any(Vec<&'static str>),
    All(Vec<&'static str>),
}

impl PermissionsCheck {
    pub fn codes(&self) -> Vec<&'static str> {
        match self {
            Self::Require(code) => vec![*code],
            Self::Any(codes) | Self::All(codes) => codes.clone(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Require(code) => format!("required permission '{code}'"),
            Self::Any(codes) => format!("any of permissions {codes:?}"),
            Self::All(codes) => format!("all permissions {codes:?}"),
        }
    }

    pub fn check(&self, user: &CurrentUser) -> bool {
        if user.is_super {
            return true;
        }

        match self {
            Self::Require(code) => user.permissions.contains(*code),
            Self::Any(codes) => codes.iter().any(|code| user.permissions.contains(*code)),
            Self::All(codes) => codes.iter().all(|code| user.permissions.contains(*code)),
        }
    }
}
