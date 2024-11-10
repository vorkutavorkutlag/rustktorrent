use std::{
    io, 
    io::prelude::*,
    str::Chars,
    fs::File,
    collections::HashMap
};

pub fn read_torrent(file_path: &str) -> io::Result<String> {
    let mut file: File = File::open(file_path)?;
    let mut byte_contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut byte_contents)?;
    // Ok(byte_contents)
    Ok(String::from_utf8_lossy(&byte_contents).to_string())
}

pub fn bdecode(mut file_str: String) {
    fn handle_dict<T, G>(mut file_chars: &[u8], index: i32) -> HashMap<T, G>{
        // d<key-value-pairs>
        // i.e. d4:spami42ee
            
    }

    fn handle_integer(mut file_str: String) -> (String, i32) {
        // i<value>e
        // i.e. i42e
        let mut integer_str: String = String::from("0");
        let mut part_len: usize = 0;
        for char in file_str.chars() {
            part_len +=1;
            match char {
                'i' => continue,
                'e' => break,
                _ => integer_str.push(char)
            }
        }
        file_str = file_str[part_len..].to_string();
        (file_str, integer_str.parse().unwrap())
    }

    fn handle_string() -> String {
        // <len>:<string>
        // i.e. 4:test
        let mut string_len: String = String::from('0');
        let mut part_len: usize = 0;
        for char in file_str.chars() {
            part_len +=1;
            match char {
                ':' => break,
                _ => string_len.push(char)
            }
        
        let mut string_len: i32 = string_len.parse().unwrap();
        let part_string: String = String::from("");
        for char in file_str.chars() {
            part_string.push(char);
            part_len +=1;
            string_len -= 1;
            if string_len <= 0 {break;}
        }
        
        file_str = file_str[part_len..].to_string();
        (file_str, integer_str.parse().unwrap())

    }
    fn handle_list<T>() -> Vec<T>{
        // l<content>e
        // i.e. l4:testi42ee
    }
    
}