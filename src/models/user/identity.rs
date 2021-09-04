use crate::error::{ApiError, Result};
use crate::models::user::UserError;

use super::Identity;

impl Identity {
    pub fn new(uid: i32, student_id: String) -> Self {
        Self {
            uid,
            student_id,
            ..Identity::default()
        }
    }

    pub async fn validate_oa_account(student_id: &str, oa_secret: &str) -> Result<()> {
        if is_default_digit(oa_secret) {
            return Err(ApiError::new(UserError::DefaultSecretDenied));
        }
        super::authserver::verify_portal_login(student_id, oa_secret).await?;
        Ok(())
    }
}

fn is_default_digit(secret: &str) -> bool {
    secret.len() == 6 && secret.chars().filter(char::is_ascii_digit).count() == 6
}

#[test]
fn test_is_ascii_digit() {
    assert!(is_default_digit("123456"));
    assert!(!is_default_digit("1234567"));
    assert!(!is_default_digit("Hello1"));
}
