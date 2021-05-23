use serde::Serialize;

pub use course::{get_current_term, is_valid_term};
pub use course::{CourseBase, CourseClass};
pub use major::{Major, PlannedCourse};

mod course;
mod major;

#[derive(Debug, Serialize)]
pub struct Course {}

#[derive(Debug, Serialize)]
pub struct CourseScore {}
