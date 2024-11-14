use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};

#[derive(Debug, Clone)]
pub enum Bencode {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<Bencode>),
    Dictionary(HashMap<String, Bencode>),
}

pub fn encode_bencode(bencode: &Bencode) -> Vec<u8> {
    match bencode {
        Bencode::Integer(i) => format!("i{}e", i).into_bytes(),
        Bencode::String(s) => {
            let mut result = format!("{}:", s.len()).into_bytes();
            result.extend_from_slice(s);
            result
        }
        Bencode::List(list) => {
            let mut result = b"l".to_vec();
            for item in list {
                result.extend(encode_bencode(item));
            }
            result.push(b'e');
            result
        }
        Bencode::Dictionary(dict) => {
            let mut result = b"d".to_vec();
            // Bencode dictionaries are sorted by key in lexicographical order
            let mut sorted_keys: Vec<&String> = dict.keys().collect();
            sorted_keys.sort();
            for key in sorted_keys {
                let key_bytes = encode_bencode(&Bencode::String(key.clone().into_bytes()));
                let value_bytes = encode_bencode(&dict[key]);
                result.extend(key_bytes);
                result.extend(value_bytes);
            }
            result.push(b'e');
            result
        }
    }
}

pub fn decode_bencode(bytes: &[u8], index: &mut usize) -> Result<Bencode, String> {
    match bytes.get(*index) {
        Some(b'i') => decode_integer(bytes, index),
        Some(b'l') => decode_list(bytes, index),
        Some(b'd') => decode_dictionary(bytes, index),
        Some(b'0'..=b'9') => decode_string(bytes, index),
        _ => Err("Invalid bencode format".to_string()),
    }
}


fn decode_integer(bytes: &[u8], index: &mut usize) -> Result<Bencode, String> {
    *index += 1; // Skip 'i'
    let start = *index;
    while let Some(&b) = bytes.get(*index) {
        if b == b'e' {
            let num_str = std::str::from_utf8(&bytes[start..*index]).map_err(|_| "Invalid UTF-8")?;
            let num = num_str.parse::<i64>().map_err(|_| "Invalid integer")?;
            *index += 1; // Skip 'e'
            return Ok(Bencode::Integer(num));
        }
        *index += 1;
    }
    Err("Unterminated integer".to_string())
}

fn decode_string(bytes: &[u8], index: &mut usize) -> Result<Bencode, String> {
    let start = *index;
    while let Some(&b) = bytes.get(*index) {
        if b == b':' {
            let len_str = std::str::from_utf8(&bytes[start..*index]).map_err(|_| "Invalid UTF-8")?;
            let len = len_str.parse::<usize>().map_err(|_| "Invalid length")?;
            *index += 1; // Skip ':'
            let end = *index + len;
            if end > bytes.len() {
                return Err("String out of bounds".to_string());
            }
            let string = bytes[*index..end].to_vec();
            *index = end;
            return Ok(Bencode::String(string));
        }
        *index += 1;
    }
    Err("Unterminated string length".to_string())
}

fn decode_list(bytes: &[u8], index: &mut usize) -> Result<Bencode, String> {
    *index += 1; // Skip 'l'
    let mut list = Vec::new();
    while let Some(&b) = bytes.get(*index) {
        if b == b'e' {
            *index += 1; // Skip 'e'
            return Ok(Bencode::List(list));
        }
        list.push(decode_bencode(bytes, index)?);
    }
    Err("Unterminated list".to_string())
}

fn decode_dictionary(bytes: &[u8], index: &mut usize) -> Result<Bencode, String> {
    *index += 1; // Skip 'd'
    let mut dict = HashMap::new();
    while let Some(&b) = bytes.get(*index) {
        if b == b'e' {
            *index += 1; // Skip 'e'
            return Ok(Bencode::Dictionary(dict));
        }
        if let Bencode::String(key_bytes) = decode_string(bytes, index)? {
            let key = String::from_utf8(key_bytes).map_err(|_| "Invalid UTF-8 in key")?;
            let value = decode_bencode(bytes, index)?;
            dict.insert(key, value);
        } else {
            return Err("Dictionary key must be a string".to_string());
        }
    }
    Err("Unterminated dictionary".to_string())
}

pub fn read_torrent_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}


