use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MarkData {
    pub slot: String,
    pub title: String,
    pub app_id: Option<String>,
}

pub type MarksState = Arc<Mutex<Vec<MarkData>>>;
