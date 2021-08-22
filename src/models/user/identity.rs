use crate::error::Result;

use super::Identity;

async fn oa_password_check(account: &str, password: &str) -> Result<()> {
    super::authserver::portal_login(account, password).await?;
    Ok(())
}

impl Identity {
    pub fn new(uid: i32, student_id: String) -> Self {
        Self {
            uid,
            student_id,
            ..Identity::default()
        }
    }

    pub async fn validate_oa_account(student_id: &str, oa_secret: &str) -> Result<()> {
        oa_password_check(student_id, oa_secret).await?;
        Ok(())
    }
}
