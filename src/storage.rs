use std::{collections::HashMap, time::Instant};

use crate::{helper_func::unpack_bulk_str, values::Value};

struct Item {
    pub value: String,
    pub created: Instant,
    pub expires: usize,
}

pub struct Storage {
    storage: HashMap<String, Item>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: Value, value: Value, expires: Value) -> Value {
        let key = unpack_bulk_str(key).unwrap();
        let value = unpack_bulk_str(value).unwrap();
        let expires = unpack_bulk_str(expires).unwrap().parse::<usize>().unwrap();
        self.storage.insert(
            key,
            Item {
                value,
                created: Instant::now(),
                expires,
            },
        );
        Value::SimpleString("OK".to_string())
    }

    pub fn get(&mut self, key: Value) -> Value {
        let key = unpack_bulk_str(key).unwrap();
        let value = self.storage.get(&key);
        match value {
            Some(v) => {
                if v.expires > 0 && v.created.elapsed().as_millis() > v.expires as u128 {
                    self.storage.remove(&key);
                    return Value::Null;
                }
                return Value::BulkString(v.value.to_string());
            }
            None => Value::Null,
        }
    }
}
