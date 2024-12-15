use std::hash::{Hash, Hasher};

pub const VERSION: &[u8] = b"HTTP/1.1";
pub const CRLF: &[u8; 2] = b"\r\n";
pub const SUPPORTED_ENCODINGS: [&str; 1] = ["gzip"];

#[derive(Debug, Ord, PartialOrd)]
pub enum Header {
    ContentEncoding(String),
    ContentType(String),
    Custom(String, String),
}

impl Header {
    pub fn name(&self) -> &str {
        match self {
            Self::ContentEncoding(_) => "Content-Encoding",
            Self::ContentType(_) => "Content-Type",
            Self::Custom(name, _) => &name[..],
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::ContentEncoding(value) | Self::ContentType(value) | Self::Custom(_, value) => {
                value
            }
        }
    }
}

impl Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::ContentEncoding(_) => 2.hash(state),
            Self::ContentType(_) => 0.hash(state),
            Self::Custom(name, _) => {
                1.hash(state);
                name.hash(state);
            }
        }
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ContentType(_), Self::ContentType(_)) => true,
            (Self::Custom(name1, _), Self::Custom(name2, _)) => name1 == name2,
            _ => false,
        }
    }
}

impl Eq for Header {}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn duplicate_headers_are_not_allowed() {
        let mut headers = HashSet::new();
        headers.insert(Header::ContentType("type1".to_string()));
        headers.insert(Header::ContentType("type2".to_string()));
        headers.insert(Header::Custom(
            "x-server".to_string(),
            "unknown".to_string(),
        ));
        headers.insert(Header::Custom("x-server".to_string(), "rust".to_string()));

        assert!(headers.contains(&Header::ContentType("type2".to_string())));
        assert!(headers.contains(&Header::Custom("x-server".to_string(), "rust".to_string())));
    }

    #[test]
    fn should_not_be_equal() {
        let header1 = Header::ContentType("type".to_string());
        let header2 = Header::Custom("x-type".to_string(), "value".to_string());

        assert_ne!(header1, header2);
    }
}
