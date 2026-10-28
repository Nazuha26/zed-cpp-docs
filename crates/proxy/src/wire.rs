use serde_json::Value;
use std::io::{self, BufRead, Read, Write};

const MAX_FRAME: usize = 64 * 1024 * 1024;
const MAX_HEADERS: usize = 16 * 1024;

pub fn read(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length = None;
    let mut total = 0;
    loop {
        let mut line = String::new();
        let count = (&mut *reader)
            .take((MAX_HEADERS + 1) as u64)
            .read_line(&mut line)?;
        if count == 0 {
            return if total == 0 {
                Ok(None)
            } else {
                Err(invalid("Truncated LSP headers"))
            };
        }
        total += count;
        if total > MAX_HEADERS {
            return Err(invalid("LSP headers exceed limit"));
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| invalid("Invalid LSP header"))?;
        if key.eq_ignore_ascii_case("Content-Length") {
            if length.is_some() {
                return Err(invalid("Duplicate Content-Length"));
            }
            length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| invalid("Invalid Content-Length"))?,
            );
        }
    }
    let length = length
        .filter(|size| *size <= MAX_FRAME)
        .ok_or_else(|| invalid("Missing or excessive Content-Length"))?;
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| invalid(&error.to_string()))
}

pub fn write(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    write!(writer, "Content-Length: {}\r\n\r\n", bytes.len())?;
    writer.write_all(&bytes)?;
    writer.flush()
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn frames_use_utf8_bytes_and_preserve_large_ids() {
        let value = serde_json::from_str(r#"{"id":9007199254740993,"result":"Привет"}"#).unwrap();
        let mut buffer = Vec::new();
        write(&mut buffer, &value).unwrap();
        write(&mut buffer, &Value::Null).unwrap();
        let mut cursor = Cursor::new(buffer);
        assert_eq!(read(&mut cursor).unwrap(), Some(value));
        assert_eq!(read(&mut cursor).unwrap(), Some(Value::Null));
        assert_eq!(read(&mut cursor).unwrap(), None);
    }

    #[test]
    fn rejects_truncated_duplicate_and_oversized_frames() {
        for input in [
            "Content-Length: 12\r\n\r\n{}",
            "Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}",
            "Content-Length: 999999999\r\n\r\n",
            "Content-Length: 2\r\n",
        ] {
            assert!(read(&mut Cursor::new(input)).is_err());
        }
    }
}
