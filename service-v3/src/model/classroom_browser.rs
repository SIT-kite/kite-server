/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use bincode::{Decode, Encode};
use chrono::NaiveDate;

#[derive(Encode, Decode, Clone, sqlx::FromRow)]
pub struct Classroom {
    /// Room number
    pub title: String,
    /// Busy time flag
    pub busy_flag: i32,
    /// Room seats
    pub capacity: Option<i32>,
}

#[derive(Debug, Encode, Decode, Clone, sqlx::FromRow)]
pub struct ClassroomQuery {
    /// The building to the classroom, for example, "一教"
    pub building: Option<String>,
    /// The region to the classroom, for example, "A", "B"
    pub region: Option<String>,
    /// Campus, for example, "徐汇校区" = 2 "奉贤校区" = 1
    pub campus: Option<i32>,
    /// Week index
    pub week: i32,
    /// Day index in a week
    pub day: i32,
    /// Want time in bits.
    ///
    /// When one bit set to 1, the corresponding course time is hoped. And the second place on the
    /// right presents the first class (8:20 - 9:55).
    /// For example, 110b to find the available classroom on class 1-2.
    pub want_time: Option<i32>,
}

/// Convert course index range string (like 1-9, 2-4) to binary, as a integer
pub fn convert_range_string_to_binary(s: &str) -> i32 {
    let mut result = 0;

    // Make sure that the time index is in 1..=11
    let validate = |x: &str| -> i32 {
        x.parse::<i32>()
            .map(|x| if (1..=11).contains(&x) { x } else { 0 })
            .unwrap_or(0)
    };

    // Set time flag by seperated time index string, like "1-2"
    let set_time_flag = |duration: &str| {
        if let Some((min, max)) = duration.split_once('-') {
            for i in validate(min)..=validate(max) {
                result |= 1 << i;
            }
        }
    };

    // Do conversion
    s.split(',').for_each(set_time_flag);
    result
}

/// Calculate week and day pair.
pub fn calculate_week_day(term_begin: NaiveDate, date_to_calculate: NaiveDate) -> (i32, i32) {
    // Calculate the days between two dates
    let gap = date_to_calculate - term_begin;
    let gap_day: i32 = gap.num_days() as i32;
    // Calculate week  and day index
    let week: i32 = gap_day / 7 + 1;
    let day: i32 = gap_day % 7 + 1;

    (week, day)
}

#[cfg(test)]
mod test {
    use super::calculate_week_day;
    use super::convert_range_string_to_binary;

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
}
