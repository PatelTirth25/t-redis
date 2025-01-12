use chrono::Utc;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

use crate::{
    rdb_encoding::{RdbEncoding, RdbString},
    storage::{Storage, StorageType},
    values::Value,
};

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

        match handle_save(&mut file, storage).await {
            Ok(()) => Value::SimpleString("OK".to_string()),
            Err(_) => Value::Null,
        }
    }
}

async fn handle_save(file: &mut File, storage: &Storage) -> Result<(), std::io::Error> {
    file.write(b"52 45 44 49 53 30 30 31 31\n").await?;
    let redis_version = RdbString::new("redis-ver".to_string()).serialize();
    let redis_version_code = RdbString::new("6.0.16".to_string()).serialize();
    file.write_all(format!("FA\n{redis_version}\n{redis_version_code}\nFE\n00\n").as_bytes())
        .await?;
    let mut buf: Vec<u8> = Vec::new();

    let mut count_exp = 0;
    let mut count_inf = 0;

    for (k, v) in storage.storage.iter() {
        match v {
            StorageType::Exp(v) => {
                if Utc::now().to_string() > v.expires {
                    continue;
                }
                count_exp += 1;
                let k = RdbString::new(k.to_string()).serialize();
                let t = RdbString::new(v.expires.to_string()).serialize();
                let v = RdbString::new(v.value.to_string()).serialize();

                buf.extend_from_slice(format!("FC\n{t}\n00\n{k}\n{v}\n").as_bytes());
            }
            StorageType::Inf(v) => {
                count_inf += 1;
                let k = RdbString::new(k.to_string()).serialize();
                let v = RdbString::new(v.to_string()).serialize();

                buf.extend_from_slice(format!("00\n{k}\n{v}\n").as_bytes());
            }
        };
    }

    let buf = String::from_utf8(buf).unwrap();
    file.write_all(format!("FB\n{count_inf}\n{count_exp}\n{buf}\n").as_bytes())
        .await?;
    Ok(())
}
