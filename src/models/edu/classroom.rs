use crate::error::Result;
use crate::models::PageView;
use chrono::prelude::*;
use chrono::ParseResult;
use sqlx::PgPool;

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct AvailClassroom {
    /// The building to the classroom
    pub building: String,
    /// Room number
    pub room: String,
    /// Busy time
    pub busy_time: i32,
    /// Room seats
    pub capacity: Option<i32>,
}

#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct AvailClassroomQuery {
    /// The building to the classroom, for example, "一教"
    pub building: Option<String>,
    /// Campus, for example, "徐汇校区" = 2 "奉贤校区" = 1
    pub campus: Option<i32>,
    /// Week index
    pub week: i32,
    /// Day index in a week
    pub day: i32,
    /// Want time in bits.
    /// When one bit set to 1, the corresponding course time is hoped. And the second place on the
    /// right presents the first class (8:20 - 9:55).
    /// For example, 110b to find the available classroom on class 1-2.
    pub want_time: Option<i32>,
}

pub async fn query_avail_classroom(
    db: &PgPool,
    query: &AvailClassroomQuery,
    page: &PageView,
) -> Result<Vec<AvailClassroom>> {
    let classrooms = sqlx::query_as(
        "SELECT building, room, busy_time::int, capacity::int FROM edu.query_available_classrooms($1, $2, $3, $4, $5)
        LIMIT $6 OFFSET $7;",
    )
    .bind(&query.campus)
    .bind(&query.building)
    .bind(query.week)
    .bind(query.day)
    .bind(query.want_time.unwrap_or(!0))
    .bind(page.count(30) as i32)
    .bind(page.offset(30) as i32)
    .fetch_all(db)
    .await?;

    Ok(classrooms)
}

pub fn convert_time_string(s: &str) -> i32 {
    let mut want_time_bits = 0;

    let check_time_index = |x: &str| -> i32 {
        if let Ok(x) = x.parse() {
            if x >= 1 && x <= 11 {
                return x;
            }
        }
        0
    };
    s.split(',').for_each(|s| {
        if let Some((min, max)) = s.split_once('-') {
            let (range_left, range_right) = (check_time_index(min), check_time_index(max));

            for i in range_left..=range_right {
                want_time_bits |= 1 << i;
            }
        }
    });
    want_time_bits
}

pub fn transform_date(s: &str) -> (i32, i32) {
    let school_term_start: NaiveDate = NaiveDate::from_ymd(2021, 9, 6);
    // Format the search date
    let fmt = "%Y-%m-%d";
    let result = NaiveDate::parse_from_str(s, fmt);
    let search_date: NaiveDate = result.unwrap();
    // Calculate the days between two dates
    let gap = search_date - school_term_start;
    let gap_day: i32 = gap.num_days() as i32;
    // Calculate weeks and days
    let week: i32 = gap_day / 7 + 1;
    let day: i32 = gap_day % 7 + 1;

    (week, day)
}

#[cfg(test)]
mod test {
    use super::convert_time_string;
    use super::transform_date;
    #[test]
    fn test_convert_time_string() {
        // Normal cases
        assert_eq!(convert_time_string("1-11"), 4094); // 1111 1111 1110
        assert_eq!(convert_time_string("1-2"), 6); // 0110
        assert_eq!(convert_time_string(""), 0);
        assert_eq!(convert_time_string("1-2,3-4"), 30); // 0001 1110
        assert_eq!(convert_time_string("1-2,5-6"), 102); // 0110 0110
        assert_eq!(convert_time_string("1-2,5-6,9-11"), 3686); // 1110 0110 0110
                                                               // Error cases
        assert_eq!(convert_time_string("1-a"), 0);
        assert_eq!(convert_time_string("1-2,1-b"), 6);
    }

    #[test]
    fn test_transform_date() {
        // Normal cases
        assert_eq!(transform_date("2021-9-17"), (2, 5));
        assert_eq!(transform_date("2021-10-23"), (7, 6));
        assert_eq!(transform_date("2021-9-23"), (3, 4));
    }
}
