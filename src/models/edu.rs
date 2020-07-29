mod major;

use serde::Serialize;

pub use major::{Major, PlannedCourse};

#[derive(Debug, Serialize)]
pub struct Course {}

#[derive(Debug, Serialize)]
pub struct CourseClass {}

#[derive(Debug, Serialize)]
pub struct CourseScore {}
