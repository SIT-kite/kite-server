use chrono::{DateTime, FixedOffset, Local, TimeZone};
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sqlx::PgPool;

use crate::error::{ApiError, Result};
use crate::models::CommonError;

const HEADER: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:75.0) Gecko/20100101 Firefox/75.0";

// 后期用strum优化
mod url {
    pub const WEATHER_NOW_FENGXIAN: &str =
        "https://devapi.qweather.com/v7/weather/now?key=f9447cf0f160412c80a05bea55b73bcc&location=101021000";
    pub const WEATHER_NOW_XUHUI: &str =
        "https://devapi.qweather.com/v7/weather/now?key=f9447cf0f160412c80a05bea55b73bcc&location=101021200";
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WeatherShow {
    /// 温度
    pub temp: i32,
    /// 天气状况
    pub weather: String,
    /// 风力等级
    pub wind_scale: i32,
    /// 当前小时累积降水量 (毫米)
    pub precipitation: f32,
}

async fn get_weather_now(url: &str) -> Result<Weather> {
    let client = Client::new();
    let response = client.get(url).header("User-Agent", HEADER).send().await?;
    let text = response.text().await?;
    let json_page: Value = serde_json::from_str(text.as_str())?;
    let result = serde_json::from_value::<Weather>(json_page["now"].clone())?;
    Ok(result)
}

async fn save_weather(db: &PgPool, data: Weather, campus: i32) -> Result<()> {
    sqlx::query(
        "INSERT INTO weather.weather_now (update_time, obs_time, temp, feel_like, weather, wind_direction, wind_scale, wind_speed, humidity, precipitation, visibility, cloud, campus)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (update_time) DO NOTHING;",
    )
        .bind(Local::now())
        .bind(data.obs_time)
        .bind(data.temp)
        .bind(data.feel_like)
        .bind(data.weather)
        .bind(data.wind_direction)
        .bind(data.wind_scale)
        .bind(data.wind_speed)
        .bind(data.humidity)
        .bind(data.precipitation)
        .bind(data.visibility)
        .bind(data.cloud)
        .bind(campus)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_weather_from_db(pool: &PgPool, campus: i32) -> Result<WeatherShow> {
    let result = sqlx::query_as(
        "SELECT temp, weather, wind_scale, precipitation FROM weather.weather_now
        WHERE campus = $1
        ORDER BY update_time DESC;",
    )
    .bind(campus)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

async fn weather_store(pool: &PgPool) -> Result<()> {
    let mut weather_urls = Vec::new();
    weather_urls.push(url::WEATHER_NOW_FENGXIAN);
    weather_urls.push(url::WEATHER_NOW_XUHUI);
    for weather_url in weather_urls {
        let weather = get_weather_now(weather_url).await?;
        let campus = match_campus(weather_url).await?;
        save_weather(pool, weather, campus).await?;
    }
    Ok(())
}

pub async fn weather_daemon(pool: PgPool) -> Result<()> {
    loop {
        // wait five minutes
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        match weather_store(&pool).await {
            Ok(_) => (),
            Err(e) => println!("Error occurred while store weather data: {}", e),
        }
    }
}

async fn match_campus(url: &str) -> Result<i32> {
    let code = url.strip_prefix(
        "https://devapi.qweather.com/v7/weather/now?key=f9447cf0f160412c80a05bea55b73bcc&location=",
    );
    if let Some(c) = code {
        match c {
            "101021000" => Ok(1),
            "101021200" => Ok(2),
            _ => Ok(0),
        }
    } else {
        Err(ApiError::new(CommonError::UrlMissed))
    }
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
    // let x = get_weather_now().await;
    // println!("{:?}", x);
}

#[test]
fn test_1() {
    let s = "https://devapi.qweather.com/v7/weather/now?key=f9447cf0f160412c80a05bea55b73bcc&location=101021000";
    let r = s.strip_prefix(
        "https://devapi.qweather.com/v7/weather/now?key=f9447cf0f160412c80a05bea55b73bcc&location=",
    );
    println!("{:?}", r);
}
