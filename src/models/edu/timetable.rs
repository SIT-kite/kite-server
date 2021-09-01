use std::collections::HashMap;
use std::ops::Add;

use chrono::{Date, DateTime, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use ics::Event as CalEvent;

use crate::bridge::Course;

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

    naive_date_time.to_string()
}

fn get_semester_start_date() -> Date<Local> {
    // TODO: Load from db or config. From config is recommended.
    Date::from_utc(NaiveDate::from_ymd(2021, 9, 6), FixedOffset::east(8))
}

fn get_semester_day_offset(start_date: Date<Local>, week_index: i32, day_in_week: i32) -> Date<Local> {
    // Assume that, the offset of the first day of the semester is 1.
    let day_offset = week_index * 7 + day_in_week;

    start_date.add(Duration::days(day_offset as i64))
}

/// Convert "12:05" to 43260
fn unchecked_time_string_to_secs_offset(time_string: &str) -> i64 {
    let (a, b) = time_string.split_once(":").unwrap();
    let (x, y) = (
        a.parse::<i64>().unwrap_or_default(),
        b.parse::<i64>().unwrap_or_default(),
    );

    x * 3600 + y * 60
}

fn get_course_start_end_time(
    campus: i32,
    building: &str,
    index_start: usize,
    index_end: usize,
) -> (NaiveTime, NaiveTime) {
    let campus = match campus {
        1 => "奉贤校区",
        2 => "徐汇校区",
        _ => unreachable!(),
    };
    let result = &SCHEDULE[campus];
    let building_table = result.get(building).unwrap_or_else(|| &result["default"]);

    let time_array = [
        NaiveTime::parse_from_str(building_table[index_start][0], "%H:%M").unwrap(),
        NaiveTime::parse_from_str(building_table[index_end][1], "%H:%M").unwrap(),
    ];
    (time_array[0], time_array[1])
}

// Campus: 1-CAMPUS_FENGXIAN, 2-CAMPUS_XUHUI
fn get_course_start_end_date_time(
    day: Date<Local>,
    campus: i32,
    building: &str,
    mut time_index: i32,
) -> (DateTime<Local>, DateTime<Local>) {
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

    let (a, b) = get_course_start_end_time(campus, building, index_start as usize, index_end as usize);

    (day.and_time(a).unwrap(), day.and_time(b).unwrap())
}

fn campus_to_i32(campus: &str) -> i32 {
    match campus {
        "奉贤校区" => 1,
        "徐汇校区" => 2,
        _ => unreachable!(),
    }
}

fn get_building_by_place(place: &str) -> String {
    match &place[..2] {
        "一教" | "二教" => "",
        _ => "default",
    }
    .to_string()
}

fn add_course_to_calendar<'a>(calendar: &mut ics::ICalendar<'a>, course: &'a Course) {
    use ics::properties::*;

    let semester_date = get_semester_start_date();
    let campus = campus_to_i32(&course.campus);
    let building = get_building_by_place(&course.place);
    let ts = get_current_dtstamp();

    // Iter in week 1 to 19
    (1..19)
        .into_iter()
        .filter(|week| course.week & (1 << week) != 0)
        .for_each(|week| {
            let uuid = uuid::Uuid::new_v4().to_string();
            let mut event = CalEvent::new(uuid, ts.clone());

            let course_date = get_semester_day_offset(semester_date, week, course.day);
            let (course_start, course_end) =
                get_course_start_end_date_time(course_date, campus, &building, course.time_index);

            event.push(Organizer::new(course.teacher.join(", ")));
            event.push(DtStart::new(course_start.naive_local().to_string()));
            event.push(DtEnd::new(course_end.naive_local().to_string()));
            event.push(Status::confirmed());
            event.push(Location::new(&course.place));
            event.push(Summary::new(&course.course_name));

            calendar.add_event(event);
        });
}

pub fn export_course_list_to_calendar(course_list: &Vec<Course>) -> Vec<u8> {
    let mut calendar = ics::ICalendar::new("2.0", "-//xyz Corp//NONSGML PDA Calendar Version 1.0//EN");
    let mut result = Vec::<u8>::new();

    for course in course_list.iter() {
        add_course_to_calendar(&mut calendar, course);
    }
    calendar.write(&mut result);

    result
}
