use std::ops::Range;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextOp {
    Equals(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumberOp {
    Equals(f64),
    GreaterThan(f64),
    LessThan(f64),
    Between(Range<f64>),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BooleanOp {
    Equals(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DateOp {
    Before(DateTime<Utc>),
    After(DateTime<Utc>),
    Between(Range<DateTime<Utc>>),
    InLast(i64),
}
