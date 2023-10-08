#![allow(dead_code)]

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::ies::ies_enum::ColumnType;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IesFile {
    pub(crate) header: IesHeader,
    pub(crate) columns: Vec<IesColumn>,
    pub(crate) rows: Vec<Vec<IesRow>>,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct IesHeader {
    pub(crate) data_offset: u32,
    pub(crate) resource_offset: u32,
    pub(crate) file_size: u32,
    pub(crate) name: String,
    pub(crate) column_count: u16,
    pub(crate) row_count: u16,
    pub(crate) number_column_count: u16,
    pub(crate) string_column_count: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IesColumn {
    pub(crate) name: String,
    pub(crate) name_second: String,
    pub(crate) column_type: ColumnType,
    pub(crate) position: u16,
}

impl Default for IesColumn {
    fn default() -> Self {
        IesColumn {
            name: "".to_string(),
            name_second: "".to_string(),
            column_type: ColumnType::Float,
            position: 0,
        }
    }
}

impl Ord for IesColumn {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.column_type, &other.column_type) {
            (ColumnType::Float, ColumnType::Float)
            | (ColumnType::String, ColumnType::String)
            | (ColumnType::StringSecond, ColumnType::StringSecond) => {
                self.position.cmp(&other.position)
            }
            (ColumnType::Float, _) => Ordering::Less,
            (_, ColumnType::Float) => Ordering::Greater,
            (ColumnType::String, ColumnType::StringSecond) => Ordering::Less,
            (ColumnType::StringSecond, ColumnType::String) => Ordering::Greater,
        }
    }
}

impl PartialOrd for IesColumn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for IesColumn {
    fn eq(&self, other: &Self) -> bool {
        self.column_type == other.column_type && self.position == other.position
    }
}

impl Eq for IesColumn {}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct IesRow {
    pub(crate) value_float: Option<f32>,
    pub(crate) value_int: Option<u32>,
    pub(crate) value_string: Option<String>,
}
