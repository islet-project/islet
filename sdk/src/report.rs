use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub buffer: Vec<u8>,
    pub user_data: Vec<u8>,
}
