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
            Self::Require(code) => format!("required capability '{code}'"),
            Self::Any(codes) => format!("any of capabilities {codes:?}"),
            Self::All(codes) => format!("all capabilities {codes:?}"),
        }
    }

    pub fn check(&self, user: &CurrentUser) -> bool {
        if user.is_super {
            return true;
        }

        match self {
            Self::Require(code) => has_permission(user, code),
            Self::Any(codes) => codes.iter().any(|code| has_permission(user, code)),
            Self::All(codes) => codes.iter().all(|code| has_permission(user, code)),
        }
    }
}

fn has_permission(user: &CurrentUser, code: &str) -> bool {
    if user.permissions.contains("*") || user.permissions.contains(code) {
        return true;
    }

    let parts: Vec<&str> = code.split(':').collect();
    for index in (1..parts.len()).rev() {
        let wildcard = format!("{}:*", parts[..index].join(":"));
        if user.permissions.contains(wildcard.as_str()) {
            return true;
        }
    }

    false
}
