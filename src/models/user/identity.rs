use crate::error::Result;

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
        super::authserver::verify_portal_login(student_id, oa_secret).await?;
        Ok(())
    }
}
