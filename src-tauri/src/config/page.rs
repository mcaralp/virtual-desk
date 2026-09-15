use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageConfig {
    pub id: String,
    pub grid: GridConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GridConfig {
    pub columns: u32,
    pub rows: u32,
    pub cells: Vec<CellConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CellConfig {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub widget: String,
}
