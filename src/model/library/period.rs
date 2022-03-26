use chrono::{Date, DateTime, Datelike, Local, Timelike};
use serde_json::json;

/// 图书馆开馆区间
static OPENING_PERIOD: &[(i32, i32); 3] = &[(830, 1130), (1300, 1600), (1730, 2100)];

/// 根据时间推算当日场次
fn get_period_index(datetime: DateTime<Local>) -> Option<i32> {
    let time = datetime.hour() * 100 + datetime.minute();

    for i in 0..OPENING_PERIOD.len() {
        let (start, end) = (OPENING_PERIOD[i].0 as u32, OPENING_PERIOD[i].1 as u32);

        if time >= start as u32 && time <= end {
            return Some((i + 1) as i32);
        }
    }
    None
}

pub fn make_period_by_datetime(datetime: DateTime<Local>) -> Option<i32> {
    // 周一、二不开放
    if datetime.weekday().num_days_from_monday() == 1 || datetime.weekday().num_days_from_monday() == 2 {
        return None;
    }

    get_period_index(datetime).map(|index| {
        datetime.year() % 100 * 100000 + datetime.month() as i32 * 1000 + datetime.day() as i32 * 10 + index
    })
}

pub fn get_current_period() -> Option<i32> {
    make_period_by_datetime(Local::now())
}

/// 根据时间推算当前（或下一场）场次
///
/// 如果当前无场次，返回下一场场次
pub fn get_next_period(datetime: DateTime<Local>) -> i32 {
    let time = datetime.hour() * 100 + datetime.minute();

    for i in 0..OPENING_PERIOD.len() {
        let end = OPENING_PERIOD[i].1 as u32;
        if time < end {
            let index = (i + 1) as i32;
            return datetime.year() % 100 * 100000
                + datetime.month() as i32 * 1000
                + datetime.day() as i32 * 10
                + index;
        }
    }
    // 如果当日无任何匹配，则下一场在明日（假设明日开馆）
    let tomorrow = datetime.date().succ();
    tomorrow.year() % 100 * 100000 + tomorrow.month() as i32 * 1000 + tomorrow.day() as i32 * 10 + 1
}

/// 获取第 i 场开放的开始时间和结束时间
pub fn get_period_range(base_date: Date<Local>, index: i32) -> (DateTime<Local>, DateTime<Local>) {
    let period_pair: (i32, i32) = OPENING_PERIOD[index as usize - 1];
    let (start, end) = (period_pair.0, period_pair.1);

    let start_ts = base_date.and_hms((start / 100) as u32, (start % 100) as u32, 0);
    let end_ts = base_date.and_hms((end / 100) as u32, (end % 100) as u32, 0);
    (start_ts, end_ts)
}
