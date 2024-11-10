use std::fs::File;
use std::io;
use std::io::prelude::*;

pub fn read_torrent(file_path: &str) -> io::Result<String> {
    let mut file: File = File::open(file_path)?;
    let mut byte_contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut byte_contents)?;
    return Ok(String::from_utf8_lossy(&byte_contents).to_string());
}

pub fn bdecode(file_str: String) {
    fn handle_dict() {
        // d<key-value-pairs>
        // i.e. d4:spami42ee
    }
    fn handle_integer() {
        // i<value>e
        // i.e. i42e 
    }
    fn handle_string () {
        // <len>:<string>
        // i.e. 4:test
    }
    fn handle_list() {
        // l<content>e
        // i.e. l4:testi42ee
    }
}