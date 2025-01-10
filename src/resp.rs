use std::io::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::values::Value;

pub struct RespHandler {
    stream: TcpStream,
    bytes: Vec<u8>,
}

impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            bytes: Vec::new(),
        }
    }

    pub async fn read_value(&mut self) -> Result<Value, std::io::Error> {
        let buf_read = self.stream.read_buf(&mut self.bytes).await?;
        if buf_read == 0 {
            return Err(Error::new(std::io::ErrorKind::NotFound, "Buffer not found"));
        }
        let res = parse_resp(&self.bytes);
        match res {
            Some((value, _)) => Ok(value),
            None => Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid data")),
        }
    }

    pub async fn write_value(&mut self, value: Value) -> Result<(), std::io::Error> {
        self.stream.write(value.serialize().as_bytes()).await?;
        Ok(())
    }
}

fn parse_resp(buf: &[u8]) -> Option<(Value, usize)> {
    match buf[0] as char {
        '+' => parse_simple_string(&buf),
        '$' => parse_bulk_string(&buf),
        '*' => parse_array(&buf),
        _ => None,
    }
}

fn parse_bulk_string(buf: &[u8]) -> Option<(Value, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buf[1..]) {
        let bulk_str_len = String::from_utf8(line.to_vec())
            .unwrap()
            .parse::<i64>()
            .unwrap();
        (bulk_str_len, len + 1)
    } else {
        return None;
    };

    let end_of_bulk_str = bulk_str_len + bytes_consumed as i64;
    let total_parsed = end_of_bulk_str + 2;

    Some((
        (Value::BulkString(Result::expect(
            String::from_utf8(buf[bytes_consumed..end_of_bulk_str as usize].to_vec()),
            "Invalid Bulk String",
        ))),
        total_parsed as usize,
    ))
}

fn parse_simple_string(buf: &[u8]) -> Option<(Value, usize)> {
    if let Some((line, len)) = read_until_crlf(&buf[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();
        return Some((Value::SimpleString(string), len + 1));
    }
    None
}

fn parse_array(buf: &[u8]) -> Option<(Value, usize)> {
    let (array_length, mut bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buf[1..]) {
        let array_length = String::from_utf8(line.to_vec())
            .unwrap()
            .parse::<i64>()
            .unwrap();
        (array_length, len + 1)
    } else {
        return None;
    };

    let mut values = Vec::new();
    for _ in 0..array_length {
        if let Some((value, len)) = parse_resp(&buf[bytes_consumed..]) {
            values.push(value);
            bytes_consumed += len;
        }
    }
    Some((Value::Array(values), buf.len()))
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    return None;
}
