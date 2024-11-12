use crate::bencode;
use std::{collections::HashMap, process};

const SHA1_LEN: usize = 20;

pub fn parse_torrent(filename: &str) {
    // hashed info == sha1 function on bencoded info key
    // announce list == announce key and announce-list key
    // piece_length == piece length key of the info key
    // size == length key of the info key or sum of all length keys in the files key in the info key
    // num_pieces == ceiiling of size / piece_length
    let info_hash: Vec<u8> = vec![0; SHA1_LEN];
    
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
    
}