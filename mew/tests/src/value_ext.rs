//! Value conversion utilities.

use mew_core::Value;

/// Trait for converting values into Row values.
pub trait IntoRowValue {
    fn into_row_value(self) -> Value;
}

impl IntoRowValue for Value {
    fn into_row_value(self) -> Value {
        self
    }
}

impl IntoRowValue for &str {
    fn into_row_value(self) -> Value {
        Value::String(self.to_string())
    }
}

impl IntoRowValue for String {
    fn into_row_value(self) -> Value {
        Value::String(self)
    }
}

impl IntoRowValue for i64 {
    fn into_row_value(self) -> Value {
        Value::Int(self)
    }
}

impl IntoRowValue for i32 {
    fn into_row_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoRowValue for usize {
    fn into_row_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoRowValue for f64 {
    fn into_row_value(self) -> Value {
        Value::Float(self)
    }
}

impl IntoRowValue for bool {
    fn into_row_value(self) -> Value {
        Value::Bool(self)
    }
}

impl<T: IntoRowValue> IntoRowValue for Option<T> {
    fn into_row_value(self) -> Value {
        match self {
            Some(v) => v.into_row_value(),
            None => Value::Null,
        }
    }
}
