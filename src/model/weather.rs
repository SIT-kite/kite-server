use chrono::{DateTime, Local};
use sqlx::PgPool;
use std::str::FromStr;

use crate::config::CONFIG;
use crate::error::Result;

const CAMPUS_FENGXIAN: i32 = 1;
const CAMPUS_XUHUI: i32 = 2;

/// 奉贤校区, campus = 1
const LOCATION_FENGXIAN: &str = "101021000";
/// 徐汇校区及其他, campus = 2
const LOCATION_XUHUI: &str = "101021200";

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct WeatherSummary {
    /// 温度
    pub temperature: i32,
    /// 天气状况
    pub weather: String,
    /// 更新时间(观测时间)
    pub ts: DateTime<Local>,
}

fn make_api_url(campus: i32) -> String {
    let location = if campus == CAMPUS_FENGXIAN {
        LOCATION_FENGXIAN
    } else {
        LOCATION_XUHUI
    };
    let key = &CONFIG.qweather_key;
    format!(
        "https://devapi.qweather.com/v7/weather/now?key={}&location={}",
        key, location
    )
}

async fn get_weather_from_qweather(campus: i32) -> Result<serde_json::Value> {
    let url = make_api_url(campus);
    let response = reqwest::get(&url).await?;

    // Note: QWeather returns data with gzip compressed, so you should add feature "gzip"
    // to reqwest library in cargo.toml.
    let text = response.text().await?;
    Ok(serde_json::Value::from_str(&text)?)
}

async fn save_weather(db: &PgPool, campus: i32, data: &serde_json::Value) -> Result<()> {
    sqlx::query("INSERT INTO weather.record (ts, campus, data) VALUES (now(), $1, $2);")
        .bind(campus)
        .bind(data)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_recent_weather(pool: &PgPool, campus: i32) -> Result<WeatherSummary> {
    let result = sqlx::query_as(
        "SELECT CAST(data->>'temp' AS integer) AS temperature, data->>'text' AS weather, CAST(data->>'obsTime' AS timestamptz) AS ts
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
    for campus_id in vec![CAMPUS_FENGXIAN, CAMPUS_XUHUI] {
        let weather = get_weather_from_qweather(campus_id).await?;
        save_weather(pool, campus_id, &weather["now"]).await?;
    }
    Ok(())
}

pub async fn weather_daemon(pool: PgPool) -> Result<()> {
    println!("Weather daemon started.");
    loop {
        if let Err(e) = update_all_weather(&pool).await {
            println!("Error occurred while update weather: {}", e);
        }
        // Wait 5 minutes.
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}
