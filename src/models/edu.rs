use serde::Serialize;

pub use classroom::{convert_time_string, query_avail_classroom, transform_campus, transform_date};
pub use classroom::{AvailClassroom, AvailClassroomQuery};
pub use course::{get_current_term, is_valid_term};
pub use course::{CourseBase, CourseClass};
pub use major::{Major, PlannedCourse};

mod classroom;
mod course;
mod major;

#[derive(Debug, Serialize)]
pub struct Course {}

#[derive(Debug, Serialize)]
pub struct CourseScore {}
