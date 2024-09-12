use std::{cmp::Ordering, collections::BTreeMap};

use ordered_float::OrderedFloat;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Number {
    PosInt(u64),
    NegInt(i64),
    Float(f64),
}

impl Number {
    fn discriminant(&self) -> usize {
        match *self {
            Number::PosInt(_) => 0,
            Number::NegInt(_) => 1,
            Number::Float(_) => 2,
        }
    }
}

impl Default for Number {
    fn default() -> Self {
        Number::PosInt(0)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::PosInt(a), Number::PosInt(b)) => a == b,
            (Number::NegInt(a), Number::NegInt(b)) => a == b,
            (Number::Float(a), Number::Float(b)) => OrderedFloat(*a) == OrderedFloat(*b),
            _ => false,
        }
    }
}

impl Eq for Number {}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Number::PosInt(a), Number::PosInt(b)) => a.cmp(b),
            (Number::NegInt(a), Number::NegInt(b)) => a.cmp(b),
            (Number::Float(a), Number::Float(b)) => OrderedFloat(*a).cmp(&OrderedFloat(*b)),
            (a, b) => a.discriminant().cmp(&b.discriminant()),
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
