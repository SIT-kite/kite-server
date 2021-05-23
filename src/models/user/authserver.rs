use super::UserError;
use crate::error::{ApiError, Result};
use actix_web::http::StatusCode;
use awc::Client;

/// Login page.
const LOGIN_URL: &str = "https://authserver.sit.edu.cn/authserver/login";

/// Concat parameters to a url-formed string.
macro_rules! make_parameter {
    // Concatenate web form parameters to a string.
    ($($para: expr => $val: expr), *) => {{
        let mut url = String::new();
        $( url = url + $para + "=" + $val + "&"; )*

        url.clone()
    }}
}

macro_rules! regex_find {
    ($text: expr, $pattern: expr) => {{
        let re = regex::Regex::new($pattern).unwrap();
        re.captures($text).map(|r| r[1].to_string())
    }};
}

/// Login on campus official auth-server with student id and password.
/// Return cookie string on `.sit.edu.cn`.
pub async fn portal_login(user_name: &str, password: &str) -> Result<String> {
    // Create a http client, but, awc::Client may not support cookie store..
    let client = Client::default();

    // Request login page to get encrypt key and so on.
    let mut response = client
        .get(LOGIN_URL)
        .send()
        .await
        .map_err(|_| ApiError::new(UserError::OaNetworkFailed))?;
    let index_html = response.body().await.unwrap();
    let cookie_string = response
        .cookies()
        .unwrap()
        .iter()
        .map(|x| format!("{}={}; ", x.name(), x.value()))
        .collect::<Vec<String>>()
        .join("");

    // Get encrypt key.
    let text = std::str::from_utf8(&index_html).unwrap();
    let aes_key = regex_find!(text, r#"var pwdDefaultEncryptSalt = "(.*?)";"#).unwrap();

    // Submit user, password, and get final token in cookies.
    let response = client
        .post(LOGIN_URL)
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .insert_header(("Referrer", LOGIN_URL))
        .insert_header(("Cookie", cookie_string))
        .send_body(&make_parameter!(
            "username" => user_name,
            "password" => &urlencoding::encode(&generate_passwd_string(&password.to_string(), &aes_key)),
            "dllt" => "userNamePasswordLogin",
            "execution" => "e1s1",
            "_eventId" => "submit",
            "rmShown" => "1",
            "lt" => &regex_find!(text, r#"<input type="hidden" name="lt" value="(.*?)"/>"#).unwrap()
        ))
        .await
        .map_err(|_| ApiError::new(UserError::OaNetworkFailed))?;
    if response.status() != StatusCode::FOUND {
        return Err(ApiError::new(UserError::OaSecretFailed));
    }
    let cookies = response.cookies().unwrap();
    let cookie_string: String = cookies
        .iter()
        .filter(|x| x.domain().unwrap_or_default() == ".sit.edu.cn")
        .map(|x| format!("{}={};", x.name(), x.value()))
        .collect::<Vec<String>>()
        .join(" ");
    Ok(cookie_string)
}

/// When submit password to `authserver.sit.edu.cn`, it's required to do AES and base64 algorithm with
/// origin password. We use a key from HTML (generated and changed by `JSESSIONID`) to help with.
pub fn generate_passwd_string(clear_password: &str, key: &str) -> String {
    use block_modes::block_padding::Pkcs7;
    use block_modes::{BlockMode, Cbc};
    type Aes128Cbc = Cbc<aes::Aes128, Pkcs7>;

    // Create an AES object.
    let cipher = Aes128Cbc::new_from_slices(key.as_bytes(), &[0u8; 16]).unwrap();
    // Concat plaintext: 64 bytes random bytes and original password.
    let mut content = Vec::new();
    content.extend_from_slice(&[0u8; 64]);
    content.extend_from_slice(clear_password.as_bytes());

    // Encrypt with AES and use do base64 encoding.
    let encrypted_passwd = cipher.encrypt_vec(&content);
    base64::encode(encrypted_passwd)
}
