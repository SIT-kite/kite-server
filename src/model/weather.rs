use std::str::FromStr;

use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::config::CONFIG;
use crate::error::Result;
use crate::model::{CAMPUS_FENGXIAN, CAMPUS_XUHUI};

#[derive(Clone, Copy)]
pub enum Campus {
    FengXian = 1,
    Xuhui = 2,
}

impl From<Campus> for String {
    fn from(campus: Campus) -> String {
        match campus {
            Campus::FengXian => "101021000",
            Campus::Xuhui => "101021200",
        }.to_string()
    }
}

#[derive(Clone, Copy)]
pub enum Language {
    SimplifiedChinese = 1,
    TraditionalChinese = 2,
    English = 3,
}

impl From<Language> for String {
    fn from(language: Language) -> String {
        match language {
            Language::SimplifiedChinese => "zh",
            Language::TraditionalChinese => "zh-hant",
            Language::English => "en",
        }.to_string()
    }
}
pub struct WeatherParam {
    pub campus: Campus,
    pub lang: Language,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct WeatherSummary {
    /// 温度
    pub temperature: i32,
    /// 天气状况
    pub weather: String,
    /// 图标 (与 app 的 assets/weather 下图标对应)
    pub icon: String,
    /// 更新时间(观测时间)
    pub ts: DateTime<Local>,
}

fn make_api_url(param: &WeatherParam) -> String {
    let location = String::from(param.campus);
    let lang = String::from(param.lang);
    let key = CONFIG.get().unwrap().qweather_key.as_str();
    format!(
        "https://devapi.qweather.com/v7/weather/now?key={}&location={}&lang={}",
        key, location, lang
    )
}

async fn get_weather_from_qweather(param: &WeatherParam) -> Result<serde_json::Value> {
    let url = make_api_url(param);
    let response = reqwest::get(&url).await?;

    // Note: QWeather returns data with gzip compressed, so you should add feature "gzip"
    // to reqwest library in cargo.toml.
    let text = response.text().await?;
    Ok(serde_json::Value::from_str(&text)?)
}

async fn save_weather(db: &PgPool, param: &WeatherParam, data: &serde_json::Value) -> Result<()> {
    sqlx::query("INSERT INTO weather.record (ts, campus, data, lang) VALUES (now(), $1, $2, $3);")
        .bind(param.campus as i32)
        .bind(data)
        .bind(param.lang as i32)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_recent_weather(pool: &PgPool, campus: i32) -> Result<WeatherSummary> {
    let result = sqlx::query_as(
        "SELECT
            CAST(data->>'temp' AS integer) AS temperature,
            data->>'text' AS weather,
            data->>'icon' AS icon,
            CAST(data->>'obsTime' AS timestamptz) AS ts
        FROM weather.record
        WHERE campus = $1
        ORDER BY record.ts DESC
        LIMIT 1;",
    )
    .bind(campus)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

async fn update_all_weather(pool: &PgPool) -> Result<()> {
    for campus in vec![Campus::FengXian, Campus::Xuhui] {
        for lang in vec![Language::SimplifiedChinese, Language::TraditionalChinese, Language::English] {
            let param = WeatherParam { campus, lang };
            let weather = get_weather_from_qweather(&param).await?;
            save_weather(pool, &param, &weather["now"]).await?;
        }
    }
    Ok(())
}

pub async fn weather_daemon(pool: PgPool) -> Result<()> {
    println!("Weather daemon started.");
    loop {
        if let Err(e) = update_all_weather(&pool).await {
            println!("Error occurred while update weather: {}", e);
        }
        // Wait 10 minutes.
        tokio::time::sleep(tokio::time::Duration::from_secs(900)).await;
    }
}
