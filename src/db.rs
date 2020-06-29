use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct CourseInfo {
    pub course_id: String,
    pub desc: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Schedule {
    pub courses: Vec<String>,
}
