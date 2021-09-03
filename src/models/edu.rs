pub use classroom::{convert_time_string, query_avail_classroom, transform_date};
pub use classroom::{AvailClassroom, AvailClassroomQuery};
pub use course::{get_current_term, is_valid_term};
pub use course::{CourseBase, CourseClass};
pub use major::{Major, PlannedCourse};
pub use timetable::export_course_list_to_calendar;

mod classroom;
mod course;
mod major;
mod timetable;

#[derive(Debug, thiserror::Error)]
pub enum EduError {}
