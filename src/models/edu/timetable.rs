use std::collections::HashMap;
use std::ops::Add;

use chrono::{Date, DateTime, Duration, FixedOffset, NaiveDateTime, NaiveTime, TimeZone, Utc};
use ics::Event as CalEvent;

use crate::bridge::Course;
use crate::models::edu::{CAMPUS_FENGXIAN, CAMPUS_XUHUI};

type ScheduleType = HashMap<&'static str, HashMap<&'static str, Vec<[&'static str; 2]>>>;

lazy_static! {
    static ref SCHEDULE: ScheduleType = {
        let mut schedule = HashMap::new();

        let mut fengxian_schedule = HashMap::new();
        fengxian_schedule.insert(
            "default",
            vec![
                ["8:20", "9:05"],
                ["9:10", "9:55"],
                ["10:15", "11:00"],
                ["11:05", "11:50"],
                ["13:00", "13:45"],
                ["13:50", "14:35"],
                ["14:55", "15:40"],
                ["15:45", "16:30"],
                ["18:00", "18:45"],
                ["18:50", "19:35"],
                ["19:40", "20:25"],
            ],
        );
        fengxian_schedule.insert(
            "一教",
            vec![
                ["8:20", "9:05"],
                ["9:10", "9:55"],
                ["10:25", "11:00"],
                ["11:05", "12:00"],
                ["13:00", "13:45"],
                ["13:50", "14:35"],
                ["14:55", "15:40"],
                ["15:45", "16:30"],
                ["18:00", "18:45"],
                ["18:50", "19:35"],
                ["19:40", "20:25"],
            ],
        );
        fengxian_schedule.insert(
            "二教",
            vec![
                ["8:20", "9:05"],
                ["9:10", "9:55"],
                ["10:15", "11:00"],
                ["11:05", "11:45"],
                ["13:00", "13:45"],
                ["13:50", "14:35"],
                ["14:55", "15:40"],
                ["15:45", "16:30"],
                ["18:00", "18:45"],
                ["18:50", "19:35"],
                ["19:40", "20:25"],
            ],
        );

        let mut xuhui_schedule = HashMap::new();
        xuhui_schedule.insert(
            "default",
            vec![
                ["8:00", "8:45"],
                ["8:50", "9:35"],
                ["9:55", "10:40"],
                ["10:45", "11:30"],
                ["13:00", "13:45"],
                ["13:50", "14:35"],
                ["14:55", "15:40"],
                ["15:45", "16:30"],
                ["18:00", "18:45"],
                ["18:50", "19:35"],
                ["19:40", "20:25"],
            ],
        );

        schedule.insert("奉贤校区", fengxian_schedule);
        schedule.insert("徐汇校区", xuhui_schedule);
        schedule
    };
}

fn get_current_dtstamp() -> String {
    let current_time = Utc::now();
    let naive_date_time = NaiveDateTime::from_timestamp(current_time.timestamp(), 0);

    naive_date_time.format("%Y%m%dT%H%M%SZ").to_string()
}

fn get_semester_start_date() -> Date<FixedOffset> {
    // TODO: Load from db or config. From config is recommended.
    FixedOffset::east(8 * 3600).ymd(2022, 2, 14)
}

/// 获取该学期的某周的第几天的时间
fn get_semester_day_offset<Tz: TimeZone>(
    start_date: &Date<Tz>, // 学期开始时间
    week_index: i32,       // 第几周，索引从1开始
    day_in_week: i32,      // 该周的第几天，周天为7
) -> Date<Tz> {
    // Assume that, the offset of the first day of the semester is 1.
    let day_offset = (week_index - 1) * 7 + day_in_week - 1;

    start_date.clone().add(Duration::days(day_offset as i64))
}

#[test]
fn test_get_semester_day_offset() {
    let offset = FixedOffset::east(8 * 3600);
    // 学期开始时间
    let start_date = offset.ymd(2021, 9, 6);

    // 第一周 周一 -> 9 月 6 日
    let day0 = get_semester_day_offset(&start_date, 1, 1);
    assert_eq!(day0, offset.ymd(2021, 9, 6));

    // 第二周 周一 -> 9 月 13 日
    let day7 = get_semester_day_offset(&start_date, 2, 1);
    assert_eq!(day7, offset.ymd(2021, 9, 13));
}

/// 获得从00:00开始到传入时间所经过的秒数
/// 注意： 该函数不会检查字符串格式是否正确
/// eg: Convert "12:05" to 43500
/// result: 12 * 3600 + 5 * 60 = 43500
fn unchecked_time_string_to_secs_offset(time_string: &str) -> i64 {
    let (a, b) = time_string.split_once(":").unwrap();
    let (x, y) = (
        a.parse::<i64>().unwrap_or_default(),
        b.parse::<i64>().unwrap_or_default(),
    );

    x * 3600 + y * 60
}

#[test]
fn test_unchecked_time_string_to_secs_offset() {
    assert_eq!(12 * 3600 + 5 * 60, unchecked_time_string_to_secs_offset("12:05"));
}

///
fn get_course_start_end_time(
    campus: i32,        // 校区
    building: &str,     // 在哪上课
    index_start: usize, // 第几节上课
    index_end: usize,   // 第几节下课
) -> (NaiveTime, NaiveTime) {
    let campus = match campus {
        CAMPUS_FENGXIAN => "奉贤校区",
        CAMPUS_XUHUI => "徐汇校区",
        _ => unreachable!(),
    };
    let result = &SCHEDULE[campus];

    // 该地点的时刻表
    let building_table = result.get(building).unwrap_or_else(|| &result["default"]);

    let time_array = [
        NaiveTime::parse_from_str(building_table[index_start - 1][0], "%H:%M").unwrap(),
        NaiveTime::parse_from_str(building_table[index_end - 1][1], "%H:%M").unwrap(),
    ];
    (time_array[0], time_array[1])
}

/// 一天上课节数使用位来控制，第n节课占用第n位,
/// 注意： 第0位空出，始终为0
fn split_start_end_index(mut time_index: i32) -> (i32, i32) {
    // When time_index = (110) in binary, it represents 1-2 in day.
    let (mut index_start, mut index_end);

    index_start = 0;
    while time_index & 1 == 0 {
        index_start += 1;
        time_index >>= 1;
    }

    // Now, index_end = 0, index_start = 1, time_index = (11) in binary.
    index_end = index_start - 1;
    while time_index & 1 == 1 {
        index_end += 1;
        time_index >>= 1;
    }
    (index_start, index_end)
}

fn get_course_start_end_date_time<Tz: TimeZone>(
    day: Date<Tz>,
    campus: i32,
    building: &str,
    time_index: (i32, i32),
) -> (DateTime<Tz>, DateTime<Tz>) {
    let (index_start, index_end) = time_index;
    let (a, b) = get_course_start_end_time(campus, building, index_start as usize, index_end as usize);

    (day.and_time(a).unwrap(), day.and_time(b).unwrap())
}

#[test]
fn test_get_course_start_end_time() {
    let campus = 1;
    let building = "default";

    let (index_start, index_end) = (1, 2);
    let (a, b) = get_course_start_end_time(campus, building, index_start, index_end);

    let hour = 3600;

    let datetime = FixedOffset::east(8 * hour).ymd(2021, 9, 1).and_time(a).unwrap();
    assert_eq!(to_datetime_string(&datetime), "20210901T002000Z".to_string());

    let datetime = FixedOffset::east(8 * hour).ymd(2021, 9, 1).and_time(b).unwrap();
    assert_eq!(to_datetime_string(&datetime), "20210901T015500Z".to_string());
}

#[test]
fn test_get_course_start_end_date_time() {}

/// 校区名转数字
fn campus_to_i32(campus: &str) -> i32 {
    match campus {
        "奉贤校区" => CAMPUS_FENGXIAN,
        "徐汇校区" => CAMPUS_XUHUI,
        _ => unreachable!(),
    }
}

/// 提取上课地点中的教学楼名称
fn get_building_by_place(place: &str) -> String {
    if place.starts_with("一教") {
        "一教".to_string()
    } else if place.starts_with("二教") {
        "二教".to_string()
    } else {
        "default".to_string()
    }
}

/// 将带时区的时间转化为ICS时间字符串
fn to_datetime_string<Tz: chrono::TimeZone>(date_time: &DateTime<Tz>) -> String {
    date_time.naive_utc().format("%Y%m%dT%H%M%SZ").to_string()
}

/// 向日程表中添加一门课
fn add_course_to_calendar<'a>(calendar: &mut ics::ICalendar<'a>, course: &'a Course, alarm_offset: i32) {
    use ics::{properties::*, Alarm};

    let semester_date = get_semester_start_date();
    let campus = campus_to_i32(&course.campus);
    let building = if campus == CAMPUS_FENGXIAN {
        get_building_by_place(&course.place)
    } else {
        "default".to_string()
    };
    let ts = get_current_dtstamp();

    // Iter in week 1 to 19
    (1..19)
        .into_iter()
        .filter(|week| course.week & (1 << week) != 0)
        .for_each(|week| {
            let course_date = get_semester_day_offset(&semester_date, week, course.day);
            let (index_start, index_end) = split_start_end_index(course.time_index);
            let (course_start, course_end) =
                get_course_start_end_date_time(course_date, campus, &building, (index_start, index_end));

            let description = if index_start == index_end {
                format!(
                    "第 {} 节\\n{}\\n{}",
                    index_start,
                    course.place,
                    course.teacher.join(", ")
                )
            } else {
                format!(
                    "第 {}-{} 节\\n{}\\n{}",
                    index_start,
                    index_end,
                    course.place,
                    course.teacher.join(", ")
                )
            };

            // 使用开始时间，地点名，课程名做md5
            let course_start_time_str = to_datetime_string(&course_start);

            let course_md5 = md5::compute(format!(
                "{}{}{}",
                course_start_time_str, course.place, course.course_name
            ));
            let course_md5 = format!("{:x}", course_md5);
            let event_uid = format!("SIT-KITE-{}", course_md5);

            let mut event = CalEvent::new(event_uid, ts.clone());

            event.push(DtStart::new(course_start_time_str));
            event.push(DtEnd::new(to_datetime_string(&course_end)));
            event.push(Location::new(&course.place));
            event.push(Summary::new(&course.course_name));
            event.push(Description::new(description.clone()));
            event.push(Status::confirmed());

            if 0 != alarm_offset {
                let alarm_time = course_start - chrono::Duration::seconds(alarm_offset as i64);
                let trigger = Trigger::new(to_datetime_string(&alarm_time));
                let alarm = Alarm::display(trigger, Description::new(description));
                event.add_alarm(alarm);
            }
            calendar.add_event(event);
        });
}

/// 导出课表到日历的日程
pub fn export_course_list_to_calendar(course_list: &[Course], alarm: i32) -> Vec<u8> {
    let mut calendar = ics::ICalendar::new(
        "2.0",
        "-//Yiban Station of Shanghai Institute of Technology//SIT-KITE//EN",
    );

    for course in course_list {
        add_course_to_calendar(&mut calendar, course, alarm);
    }

    let mut result = Vec::<u8>::new();
    calendar.write(&mut result).unwrap();

    result
}

#[test]
fn test_export_course_list_to_calendar() {
    let course = Course {
        course_name: "Test".to_string(),
        day: 1,
        time_index: 0b1110,
        week: 0b111,
        place: "一教A101".to_string(),
        teacher: vec!["Hello".to_string(), "World".to_string()],
        campus: "奉贤校区".to_string(),
        credit: 0.0,
        hours: 2,
        dyn_class_id: "".to_string(),
        course_id: "".to_string(),
    };
    let export = export_course_list_to_calendar(&[course], 15);
    let s = String::from_utf8(export).unwrap();
    println!("{}", s);
}

pub fn generate_sign(uid: i32) -> String {
    use crate::config::CONFIG;

    let s = format!("{}-5eb63bbbe01eeed0-{}", CONFIG.server.secret, uid);
    let hashed_s = md5::compute(s);
    format!("{:x}", hashed_s)
}
