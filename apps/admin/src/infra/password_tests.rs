use super::PasswordUtils;

#[test]
fn password_hashing_and_verification() {
    let password = "test_password_123";
    let hash = PasswordUtils::hash_password(password).expect("hash password");

    assert!(!hash.is_empty());
    assert!(PasswordUtils::verify_password(password, &hash));
    assert!(!PasswordUtils::verify_password("wrong_password", &hash));
    assert!(!PasswordUtils::verify_password(password, "invalid_hash"));
}

#[test]
fn different_passwords_produce_different_hashes() {
    let hash1 = PasswordUtils::hash_password("password1").expect("hash first password");
    let hash2 = PasswordUtils::hash_password("password2").expect("hash second password");

    assert_ne!(hash1, hash2);
}

#[test]
fn random_salt_changes_the_hash_for_the_same_password() {
    let password = "same_password";
    let hash1 = PasswordUtils::hash_password(password).expect("hash password");
    let hash2 = PasswordUtils::hash_password(password).expect("hash password again");

    assert_ne!(hash1, hash2);
    assert!(PasswordUtils::verify_password(password, &hash1));
    assert!(PasswordUtils::verify_password(password, &hash2));
}
