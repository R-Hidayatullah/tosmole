#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) enum ColumnType {
    Float,
    String,
    StringSecond,
}
