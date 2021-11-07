use chrono::{DateTime, FixedOffset, Local, TimeZone};
use reqwest::Client;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::error::Result;

const HEADER: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:75.0) Gecko/20100101 Firefox/75.0";

mod url {
    pub const WEATHER_NOW: &str =
        "https://devapi.qweather.com/v7/weather/now?key=f9447cf0f160412c80a05bea55b73bcc&location=101021000";
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    #[serde(rename(deserialize = "obsTime"), deserialize_with = "str_to_date_time")]
    /// 观测时间
    pub obs_time: DateTime<Local>,
    #[serde(rename(deserialize = "temp"), deserialize_with = "str_to_i32")]
    /// 温度
    pub temp: i32,
    #[serde(rename(deserialize = "feelsLike"), deserialize_with = "str_to_i32")]
    /// 体感温度
    pub feel_like: i32,
    #[serde(rename(deserialize = "text"))]
    /// 天气状况
    pub weather: String,
    #[serde(rename(deserialize = "windDir"))]
    /// 风向
    pub wind_direction: String,
    #[serde(rename(deserialize = "windScale"), deserialize_with = "str_to_i32")]
    /// 风力等级
    pub wind_scale: i32,
    #[serde(rename(deserialize = "windSpeed"), deserialize_with = "str_to_f32")]
    /// 风速
    pub wind_speed: f32,
    #[serde(rename(deserialize = "humidity"), deserialize_with = "str_to_f32")]
    /// 相对湿度
    pub humidity: f32,
    #[serde(rename(deserialize = "precip"), deserialize_with = "str_to_f32")]
    /// 当前小时累积降水量 (毫米)
    pub precipitation: f32,
    #[serde(rename(deserialize = "vis"), deserialize_with = "str_to_i32")]
    /// 能见度
    pub visibility: i32,
    #[serde(rename(deserialize = "cloud"), deserialize_with = "str_to_f32")]
    /// 云量
    pub cloud: f32,
}

pub async fn get_weather_now() -> Result<Weather> {
    let client = Client::new();
    let response = client
        .get(url::WEATHER_NOW)
        .header("User-Agent", HEADER)
        .send()
        .await?;
    let text = response.text().await?;
    let json_page: Value = serde_json::from_str(text.as_str())?;
    let result = serde_json::from_value::<Weather>(json_page["now"].clone())?;
    Ok(result)
}

fn parse_date_time(date_time: &str) -> DateTime<Local> {
    let tz = FixedOffset::east(8 * 3600);
    let dt = tz
        .datetime_from_str(date_time, "%Y-%m-%dT%H:%M+08:00")
        .unwrap_or_else(|_| tz.timestamp_nanos(0));

    DateTime::<Local>::from(dt)
}

pub fn str_to_f32<'de, D>(deserializer: D) -> std::result::Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let i = s.parse::<f32>().unwrap_or(-1.0);
    Ok(i)
}

pub fn str_to_i32<'de, D>(deserializer: D) -> std::result::Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let i = s.parse::<i32>().unwrap_or_default();
    Ok(i)
}

pub fn str_to_date_time<'de, D>(deserializer: D) -> std::result::Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let r = parse_date_time(&s);
    Ok(r)
}

#[tokio::test]
async fn test() {
    let x = get_weather_now().await;
    println!("{:?}", x);
}

#[test]
fn test_1() {}
