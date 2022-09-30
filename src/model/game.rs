use crate::error::Result;
use crate::util::deserialize_from_str;
use chrono::{DateTime, Local};
use sqlx::PgPool;
use substring::Substring;

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct PublicGameRecord {
    /// 学号
    #[serde(rename = "studentId")]
    pub student_id: String,
    /// 姓名
    #[serde(skip_serializing)]
    pub name: String,
    /// 成绩
    pub score: i32,
}

#[derive(serde::Deserialize)]
pub struct GameRecord {
    /// Timestamp timezone
    #[serde(rename = "dateTime", deserialize_with = "deserialize_from_str")]
    pub ts: DateTime<Local>,
    /// Game type
    pub game: i32,
    /// Score
    pub score: i32,
}

/// 映射学院名称. 输入学号
fn map_school(student_id: &str) -> &str {
    if student_id.len() == 9 {
        map_post_graduate_school(student_id)
    } else {
        map_under_graduate_school(student_id)
    }
}

/// 研究生学号映射规则
fn map_post_graduate_school(student_id: &str) -> &str {
    let index: i32 = student_id[3..5].parse().unwrap_or_default();

    match index {
        06 => "化工",
        07 => "香料",
        08 => "材料",
        09 => "机械",
        10 => "电气",
        11 => "生态",
        12 => "经管",
        13 => "城建",
        14 => "计算机",
        15 => "轨交",
        16 => "人文",
        17 => "艺术",
        18 => "理学院",
        _ => "未知",
    }
}

/// 本科生学号映射规则
fn map_under_graduate_school(student_id: &str) -> &str {
    let index: i32 = student_id[3..5].parse().unwrap_or_default();

    match index {
        1 => "材料",
        2 => "机械",
        3 => "电气",
        4 => "计算机",
        5 | 6 => "城建",
        7 => "化工",
        8 => "香料",
        9 => "艺术",
        10 => "经管",
        11 => "外国语",
        14 => "生态",
        15 => "轨交",
        16 => "化妆品",
        21 => "人文",
        22 => "理学院",
        24 => "工创",
        _ => "未知",
    }
}

/// 学号/工号生成字符串
fn map_student_id(student_id: &str, name: &str) -> String {
    if student_id.len() == 9 || student_id.len() == 10 {
        let grade = &student_id[..2];
        let school = map_school(student_id);
        format!("{} {}{}同学", grade, school, name.substring(0, 1))
    } else if student_id.len() == 4 {
        format!("{}** {}老师", &student_id[..2], name.substring(0, 1))
    } else {
        String::from("未知用户")
    }
}

pub async fn get_ranking(
    pool: &PgPool,
    student_id: Option<String>,
    game: i32,
    after: String,
) -> Result<Vec<PublicGameRecord>> {
    let ranking: Vec<PublicGameRecord> = sqlx::query_as(
        "SELECT student_id, name, score
        FROM \"user\".account,
        (SELECT account AS student_id, MAX(score) AS score
        FROM game.record, \"user\".account
        WHERE record.uid = account.uid
                    AND game = $1 And ts>= to_date($2, 'YYYY-MM-DD')
                GROUP BY student_id) rank
        WHERE account.account = rank.student_id
        ORDER BY score DESC;",
    )
    .bind(game)
    .bind(after)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|e: PublicGameRecord| {
        let student = map_student_id(&e.student_id, &e.name);
        let student = if student_id.is_some() && student_id.as_ref().unwrap().eq(&e.student_id) {
            format!("{} (我)", student)
        } else {
            student
        };

        PublicGameRecord {
            student_id: student,
            name: "".to_string(),
            score: e.score,
        }
    })
    .collect();

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
