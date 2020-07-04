use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct CourseInfo {
    pub course_id: String,
    pub desc: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Schedule {
    pub username: String,
    pub courses: Vec<String>,
}
