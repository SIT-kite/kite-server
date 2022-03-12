use crate::error::Result;
use chrono::{DateTime, Local};
use sqlx::PgPool;

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct PublicGameRecord {
    /// 学号 (最后 3 位隐去）
    #[serde(rename = "studentId")]
    pub student_id: String,
    /// 成绩
    pub score: i32,
}

#[derive(serde::Deserialize)]
pub struct GameRecord {
    /// Timestamp timezone
    pub ts: DateTime<Local>,
    /// Game type
    pub game: i32,
    /// Score
    pub score: i32,
}

pub async fn get_ranking(pool: &PgPool, game: i32, count: i32) -> Result<Vec<PublicGameRecord>> {
    let ranking = sqlx::query_as(
        "SELECT (LEFT(account, -3) | '***') AS student_id, MAX(score) AS score
        FROM game.record, \"user\".account
        WHERE record.uid = account.uid
            AND game = $1
            AND ts >= current_date
        GROUP BY student_id
        ORDER BY score DESC
        LIMIT $2;",
    )
    .bind(game)
    .bind(count)
    .fetch_all(pool)
    .await?;

    Ok(ranking)
}

pub async fn post_record(pool: &PgPool, uid: i32, new_record: GameRecord) -> Result<()> {
    sqlx::query("INSERT INTO game.record(ts, uid, game, score) VALUES($1, $2, $3, $4);")
        .bind(new_record.ts)
        .bind(uid)
        .bind(new_record.game)
        .bind(new_record.score)
        .execute(pool)
        .await?;

    Ok(())
}
