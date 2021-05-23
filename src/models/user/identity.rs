use super::Identity;
use crate::error::Result;

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

    pub fn validate_identity_number(identity_number: &str) -> bool {
        let re = regex::Regex::new("[0-9]{5}[0-9X]").unwrap();
        re.is_match(identity_number) && identity_number.len() == 6
    }
}
