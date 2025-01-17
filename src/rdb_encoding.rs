pub trait RdbEncoding {
    fn serialize(&self) -> String;
    fn deserialize(&self) -> Result<String, std::io::Error>;
}

pub struct RdbSize(i128);
pub struct RdbString(String);

impl RdbSize {
    pub fn new(v: i128) -> Self {
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
        self.0.to_string() + "\n"
    }
    fn deserialize(&self) -> Result<String, std::io::Error> {
        Ok(self.0.to_string())
    }
}

impl RdbEncoding for RdbString {
    fn serialize(&self) -> String {
        let l = self.0.to_string().len();
        l.to_string() + "\r" + &self.0.to_string() + "\n"
    }
    fn deserialize(&self) -> Result<String, std::io::Error> {
        match self.0.to_string().split_once("\r") {
            Some((v, _)) => Ok(v.to_string()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid rdb string",
            )),
        }
    }
}
