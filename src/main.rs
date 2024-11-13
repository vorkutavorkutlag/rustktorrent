use std::{process};
mod bencode;
mod torrent_reader;

fn main() {
    let filename: &str = "sonic.torrent";
    let (infohash, announce_list, piece_length, size, num_pieces, pieces) = match torrent_reader::parse_torrent(filename) {
        Ok(values) => values,
        Err(e) => {
            eprintln!("Error parsing torrent: {}", e);
            process::exit(1);}
    };

    println!("{:#?}", (infohash, announce_list, piece_length, size, num_pieces))
    
}