use super::{Identity, UserError};
use crate::error::{ApiError, Result};
use actix_http::http::StatusCode;
use serde::Serialize;

async fn oa_password_check(account: &String, password: &String) -> Result<bool> {
    use actix_web::client;

    #[derive(Serialize)]
    struct RequestPayload {
        pub code: String,
        pub pwd: String,
    }
    if let Ok(mut web_client) = client::Client::new()
        .post("http://210.35.96.114/report/report/ssoCheckUser")
        .set_header("Referer", "http://xgfy.sit.edu.cn/h5/")
        .send_json(&RequestPayload {
            code: account.clone(),
            pwd: password.clone(),
        })
        .await
    {
        if web_client.status() == StatusCode::OK {
            let body = web_client.body().await?;
            return Ok(body.as_ref() == r#"{"code":0,"msg":null,"data":true}"#.as_bytes());
        }
    }
    Err(ApiError::new(UserError::OaNetworkFailed))
}

impl Identity {
    pub fn new(uid: i32, student_id: &String) -> Self {
        Self {
            uid,
            student_id: student_id.clone(),
            ..Identity::default()
        }
    }

    pub async fn validate_oa_account(student_id: &String, oa_secret: &String) -> Result<bool> {
        oa_password_check(student_id, oa_secret).await
    }

    pub fn validate_identity_number(identity_number: &str) -> bool {
        // Commented on July 22, 2020.
        // Due to changes in business requirements, a complete ID number is no longer required.
        //
        // let magic_array = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
        // let tail_chars = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
        // let mut sum: usize = 0;
        //
        // if identity_number.len() != 18 {
        //     return false;
        // }
        // for i in 0..17 {
        //     sum += magic_array[i] as usize * (identity_number[i] - '0' as u8) as usize;
        // }
        // return identity_number[17] as char == (if sum % 11 != 2 { tail_chars[sum % 11] } else { 'X' });

        let re = regex::Regex::new("[0-9]{5}[0-9X]").unwrap();
        return re.is_match(identity_number) && identity_number.len() == 6;
    }
}

#[cfg(test)]
mod test {
    // Commented on July 22, 2020.
    // Due to changes in business requirements, a complete ID number is no longer required.
    //
    // #[test]
    // pub fn test_identity_number_validation() {
    //     assert_eq!(
    //         true,
    //         super::Identity::validate_identity_number("110101192007156996".as_bytes())
    //     );
    //     assert_eq!(
    //         false,
    //         super::Identity::validate_identity_number("random_string".as_bytes())
    //     );
    //     assert_eq!(
    //         true,
    //         super::Identity::validate_identity_number("210202192007159834".as_bytes())
    //     );
    // }
}
