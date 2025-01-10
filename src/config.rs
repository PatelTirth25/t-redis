use std::fs::OpenOptions;

use crate::values::Value;

#[derive(Debug)]
pub struct Config {
    dir: String,
    dbfilename: String,
}

impl Config {
    pub fn new(dir: &str, dbfilename: &str) -> Config {
        let _file = OpenOptions::new()
            .create_new(true)
            .open(dir.to_string() + dbfilename);
        Config {
            dir: dir.to_string(),
            dbfilename: dbfilename.to_string(),
        }
    }
    pub fn dir(&self) -> Value {
        Value::Array(vec![
            Value::BulkString("dir".to_string()),
            Value::BulkString(self.dir.clone()),
        ])
    }
    pub fn dbfilename(&self) -> Value {
        Value::Array(vec![
            Value::BulkString("dbfilename".to_string()),
            Value::BulkString(self.dbfilename.clone()),
        ])
    }
}
