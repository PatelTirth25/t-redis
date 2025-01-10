#[derive(Clone, Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    Null,
}
impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Value::Array(v) => {
                let mut s = String::new();
                s.push_str("*");
                s.push_str(&v.len().to_string());
                s.push_str("\r\n");
                for i in v {
                    s.push_str(&i.serialize());
                }
                s
            }
            Value::Null => format!("$-1\r\n"),
            // _ => panic!("Unsupported value for serialize"),
        }
    }
}
