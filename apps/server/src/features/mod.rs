pub mod account;
pub mod auth;
pub mod dashboard;
pub mod manage;
pub mod system;

#[cfg(test)]
mod tests {
    use super::account::service::AccountService;
    use crate::infra::password::PasswordUtils;

    #[test]
    fn account_password_change_requires_current_password_and_confirmation() {
        let current_hash = PasswordUtils::hash_password("current-password").expect("hash");

        let new_hash = AccountService::build_password_hash(
            "current-password",
            &current_hash,
            "new-password",
            "new-password",
        )
        .expect("password hash");

        assert!(PasswordUtils::verify_password("new-password", &new_hash));
        assert!(!PasswordUtils::verify_password("current-password", &new_hash));
        assert!(
            AccountService::build_password_hash(
                "wrong-password",
                &current_hash,
                "new-password",
                "new-password",
            )
            .is_err()
        );
        assert!(
            AccountService::build_password_hash(
                "current-password",
                &current_hash,
                "new-password",
                "different-password",
            )
            .is_err()
        );
    }
}
