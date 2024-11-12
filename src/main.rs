use std::{process};

use bencode::Bencode;
mod bencode;
mod torrent_reader;

fn main() {
    let filename: &str = "sonic.torrent";
    
    // match torrent_hashmap {
    //     Bencode::Dictionary(ref dict) => {
    //         for (key, value) in dict.iter() {
    //             println!("Key: {}, Value: {:?}", key, value);
    //         }
    //     }
    //     _ => {
    //         eprintln!("Decoded data is not a dictionary");
    //         process::exit(1);
    //     }
    // }
}