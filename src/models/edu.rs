mod class;
mod major;

use serde::Serialize;

pub use class::get_current_term;
pub use class::CourseClass;
pub use major::{Major, PlannedCourse};

#[derive(Debug, Serialize)]
pub struct Course {}

#[derive(Debug, Serialize)]
pub struct CourseScore {}
