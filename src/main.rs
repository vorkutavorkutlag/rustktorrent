use std::{process};

use bencode::Bencode;
mod bencode;

fn main() {
    let filename = "sonic.torrent";
    let torrent_hashmap = match bencode::read_torrent_file(filename) {
        Ok(bytes) => {
            let mut index = 0;
            match bencode::decode_bencode(&bytes, &mut index) {
                Ok(decoded) => decoded,
                Err(e) => {eprintln!("Error decoding bencode: {}", e); process::exit(1)},
            }
        }
        Err(e) => {eprintln!("Error reading file: {}", e); process::exit(1)},
    };
    
    match torrent_hashmap {
        Bencode::Dictionary(ref dict) => {
            for (key, value) in dict.iter() {
                println!("Key: {}, Value: {:?}", key, value);
            }
        }
        _ => {
            eprintln!("Decoded data is not a dictionary");
            process::exit(1);
        }
    }
}