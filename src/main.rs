use std::{process, str};
use tokio::sync::mpsc;
use uuid::Uuid;

mod bencode;
mod torrent_reader;
mod tracker;

#[tokio::main]
async fn main() {
    const BRAND: & str = "RK";
    const VERSION: &str = "01";
    let uuid = &format!("{}", Uuid::new_v4())[..16];
    
    let mut session_uuid: String = String::new();
    session_uuid.push_str(BRAND);
    session_uuid.push_str(VERSION);
    session_uuid.push_str(uuid);

    let filename: &str = "test.torrent";
    let 
      (infohash,
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
    let downloaded: u64 = 0;

    // temporary arbitrary buffer. should research more and see how big communication actually gets
    let (tracker_tx, mut tracker_rx) = mpsc::channel(1024);

    tracker::start_tracker_comm(infohash, announce_list, size, session_uuid, downloaded, tracker_tx).await;

    while let Some(peer) = tracker_rx.recv().await {
      println!("Main received peer: {:#?}", peer);
  }
    
}