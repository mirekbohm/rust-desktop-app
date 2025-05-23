use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub date: DateTime<Local>,
}

pub struct DataStore {
    data: Vec<TableData>,
    next_id: u32,
}

impl DataStore {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_sample_data(&mut self) {
        let sample_names = vec![
            "Alpha", "Beta", "Gamma", "Delta", "Epsilon",
            "Zeta", "Eta", "Theta", "Iota", "Kappa"
        ];
        
        for name in sample_names.iter().take(5) {
            self.data.push(TableData {
                id: self.next_id,
                name: name.to_string(),
                value: fastrand::f64() * 1000.0,
                date: Local::now(),
            });
            self.next_id += 1;
        }
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
        self.next_id = 1;
    }

    pub fn get_all_data(&self) -> &Vec<TableData> {
        &self.data
    }

    pub fn get_record_count(&self) -> usize {
        self.data.len()
    }
}
