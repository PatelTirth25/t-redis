use chrono::Utc;
use std::io::Error;

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    rdb_encoding::{RdbEncoding, RdbSize, RdbString},
    storage::{Item, Storage, StorageType},
    values::Value,
};

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

pub async fn handle_save(file: &mut File, storage: &Storage) -> Result<(), std::io::Error> {
    let redis_heading = RdbString::new("REDIS0011".to_string()).serialize();
    let redis_version = RdbString::new("redis-ver".to_string()).serialize();
    let redis_version_code = RdbString::new("6.0.16".to_string()).serialize();
    file.write_all(
        format!("{redis_heading}FA\n{redis_version}{redis_version_code}FE\n00\n").as_bytes(),
    )
    .await?;
    let mut buf = String::new();

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

                buf.push_str(format!("FC\n{t}00\n{k}{v}").as_str());
            }
            StorageType::Inf(v) => {
                count_inf += 1;
                let k = RdbString::new(k.to_string()).serialize();
                let v = RdbString::new(v.to_string()).serialize();

                buf.push_str(format!("00\n{k}{v}").as_str());
            }
        };
    }

    let count_exp = RdbSize::new(count_exp as i128).serialize();
    let count_inf = RdbSize::new(count_inf as i128).serialize();
    file.write_all(format!("FB\n{count_inf}{count_exp}{buf}").as_bytes())
        .await?;
    Ok(())
}

pub async fn load_rdb(
    storage: &mut Storage,
    dir: &str,
    dbfilename: &str,
) -> Result<(), std::io::Error> {
    let mut file = File::open(format!("{dir}{dbfilename}")).await?;
    let mut buf: Vec<u8> = Vec::new();
    let _ = file.read_to_end(&mut buf).await?;
    let buf = String::from_utf8(buf).unwrap();
    let mut buf = buf.split('\n').collect::<Vec<&str>>();
    let _ = buf.remove(buf.len() - 1);
    println!("{:#?}", buf);

    println!("Redis RDB file headers and metadata...");
    println!("{}", RdbString::new(buf[0].to_string()).deserialize()?);
    println!("{}", buf[1]);
    println!("{}", RdbString::new(buf[2].to_string()).deserialize()?);
    println!("{}", RdbString::new(buf[3].to_string()).deserialize()?);

    let mut i = 4;
    while i < buf.len() {
        if buf[i] == "FE" {
            i += 1;
            println!(
                "DB index: {}",
                RdbSize::new(buf[i].parse::<i128>().unwrap()).deserialize()?
            );
            i += 1;
            println!("{}", buf[i]);

            let mut cnt = 0;
            i += 1;
            cnt += buf[i].parse::<i128>().unwrap();

            i += 1;
            cnt += buf[i].parse::<i128>().unwrap();

            i += 1;

            while cnt > 0 {
                if buf[i] == "FC" {
                    i += 1;
                    println!(
                        "Expires: {:#?}, key: {:#?}, Value: {:#?}, {:#?}",
                        buf[i],
                        buf[i + 2],
                        buf[i + 3],
                        buf[i + 1]
                    );
                    let _ = storage.set(
                        StorageType::Exp(Item {
                            value: RdbString::new(buf[i + 3].to_string()).deserialize()?,
                            expires: RdbString::new(buf[i].to_string()).deserialize()?,
                        }),
                        Value::BulkString(RdbString::new(buf[i + 2].to_string()).deserialize()?),
                    );
                    i += 4;
                    cnt -= 1;
                } else if buf[i] == "00" {
                    i += 1;
                    println!("Key: {:#?}, Value: {:#?}", buf[i], buf[i + 1]);

                    let _ = storage.set(
                        StorageType::Inf(RdbString::new(buf[i + 1].to_string()).deserialize()?),
                        Value::BulkString(RdbString::new(buf[i].to_string()).deserialize()?),
                    );
                    i += 2;
                    cnt -= 1;
                }
            }
        }
    }

    Ok(())
}
