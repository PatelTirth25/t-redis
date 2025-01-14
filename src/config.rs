use std::io::SeekFrom;

use tokio::{fs::OpenOptions, io::AsyncSeekExt};

use crate::{helper_func::handle_save, storage::Storage, values::Value};

#[derive(Debug)]
pub struct Config {
    dir: String,
    dbfilename: String,
}

impl Config {
    pub fn new(dir: &str, dbfilename: &str) -> Config {
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

    pub async fn save(&self, storage: &Storage) -> Value {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.dir.clone() + self.dbfilename.as_str())
            .await
            .unwrap();
        file.set_len(0).await.unwrap();
        file.seek(SeekFrom::Start(0)).await.unwrap();

        match handle_save(&mut file, storage).await {
            Ok(()) => Value::SimpleString("OK".to_string()),
            Err(_) => Value::Null,
        }
    }
}
