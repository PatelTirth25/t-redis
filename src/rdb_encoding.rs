pub trait RdbEncoding {
    fn serialize(&self) -> String;
}

pub struct RdbSize(String);
pub struct RdbString(String);

impl RdbSize {
    pub fn new(v: String) -> Self {
        Self(v)
    }
}

impl RdbString {
    pub fn new(v: String) -> Self {
        Self(v)
    }
}

impl RdbEncoding for RdbSize {
    fn serialize(&self) -> String {
        let mut v: Vec<u8> = Vec::new();
        v.push(self.0.len() as u8);

        let formatted = v
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        formatted
    }
}

impl RdbEncoding for RdbString {
    fn serialize(&self) -> String {
        let mut v: Vec<u8> = Vec::new();
        v.push(self.0.len() as u8);
        v.extend_from_slice(self.0.as_bytes());
        let formatted = v
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        formatted
    }
}
