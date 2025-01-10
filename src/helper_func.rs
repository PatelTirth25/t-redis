use std::io::Error;

use crate::values::Value;

pub fn extract_command(value: Value) -> Result<(String, Vec<Value>), std::io::Error> {
    match value {
        Value::SimpleString(s) => Ok((s, vec![])),
        Value::BulkString(s) => Ok((s, vec![])),
        Value::Array(s) => Ok((
            unpack_bulk_str(s.first().unwrap().clone())?,
            s.into_iter().skip(1).collect(),
        )),
        _ => Err(Error::new(std::io::ErrorKind::Other, "Unexpected value")),
    }
}

pub fn unpack_bulk_str(value: Value) -> Result<String, std::io::Error> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(Error::new(
            std::io::ErrorKind::Other,
            "Unexpected Bulk String",
        )),
    }
}
