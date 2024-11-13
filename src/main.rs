use std::{process};

use bencode::Bencode;
mod bencode;
mod torrent_reader;

fn main() {
    let filename: &str = "sonic.torrent";
    let (hashed_info, announce_list, piece_length, size, num_pieces) = match torrent_reader::parse_torrent(filename) {
        Ok(values) => values,
        Err(e) => {
            eprintln!("Error parsing torrent: {}", e);
            process::exit(1);}
    };
    println!("{:#?}", (hashed_info, announce_list, piece_length, size, num_pieces))
    
}