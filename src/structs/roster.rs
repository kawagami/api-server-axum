use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RosterRequest {
    pub names: Vec<String>,
    pub days: u32,
    pub rule: String,
}

#[derive(Serialize)]
pub struct StaffShift {
    pub id: usize,
    pub name: String,
    pub shifts: Vec<String>,
}

#[derive(Serialize)]
pub struct RosterResponse {
    pub status: String,
    pub data: Vec<StaffShift>,
}
