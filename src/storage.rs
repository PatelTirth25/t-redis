use std::collections::HashMap;

use chrono::Utc;
use regex::Regex;

use crate::{helper_func::unpack_bulk_str, values::Value};

pub struct Item {
    pub value: String,
    pub expires: String,
}

pub struct Storage {
    pub storage: HashMap<String, StorageType>,
}

pub enum StorageType {
    Exp(Item),
    Inf(String),
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
    pub fn set(&mut self, data: StorageType, key: Value) -> Value {
        let key = unpack_bulk_str(key).unwrap();
        match data {
            StorageType::Exp(_) => {
                self.storage.insert(key, data);
                return Value::SimpleString("OK".to_string());
            }
            StorageType::Inf(_) => {
                self.storage.insert(key, data);
                return Value::SimpleString("OK".to_string());
            }
        }
    }

    pub fn get(&mut self, key: Value) -> Value {
        let key = unpack_bulk_str(key).unwrap();
        let value = self.storage.get(&key);
        match value {
            Some(v) => match v {
                StorageType::Exp(v) => {
                    if Utc::now().to_string() > v.expires {
                        self.storage.remove(&key);
                        return Value::Null;
                    }
                    return Value::BulkString(v.value.to_string());
                }
                StorageType::Inf(v) => {
                    return Value::BulkString(v.to_string());
                }
            },
            None => Value::Null,
        }
    }

    pub fn keys(&self, pattern: Value) -> Value {
        let pattern = unpack_bulk_str(pattern).unwrap();
        let regex = Regex::new(&pattern);

        match regex {
            Ok(r) => {
                let mut keys = Vec::new();
                for key in self.storage.keys() {
                    match self.storage.get(key) {
                        Some(v) => match v {
                            StorageType::Exp(v) => {
                                if Utc::now().to_string() > v.expires {
                                    continue;
                                } else if r.is_match(key) {
                                    keys.push(Value::BulkString(key.to_string()));
                                }
                            }
                            StorageType::Inf(_) => {
                                if r.is_match(key) {
                                    keys.push(Value::BulkString(key.to_string()));
                                }
                            }
                        },
                        None => {}
                    }
                }
                Value::Array(keys)
            }
            Err(_) => Value::Null,
        }
    }
}
