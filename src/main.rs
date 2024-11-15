use std::{process};
use uuid::Uuid;

mod bencode;
mod torrent_reader;
mod tracker;

fn main() {
    const VERSION: &str = "01";
    const BRAND: & str = "RK";
    let uuid = Uuid::new_v4().to_bytes_le();
    
    let mut session_uuid: Vec<u8> = Vec::new();
    session_uuid.extend_from_slice(&BRAND.as_bytes());
    session_uuid.extend_from_slice(&VERSION.as_bytes());
    session_uuid.extend_from_slice(&uuid);

    let filename: &str = "sonic.torrent";
    let (infohash,
         announce_list,
         piece_length,
         size,
         num_pieces,
         pieces) = match torrent_reader::parse_torrent(filename) {
        
        Ok(values) => values,
        Err(e) => {
            eprintln!("Error parsing torrent: {}", e);
            process::exit(1);}
    };

    let downloaded: i64 = 0;
    tracker::start_tracker_comm(infohash, announce_list, size, session_uuid, downloaded)
    
}