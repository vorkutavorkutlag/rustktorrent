use std::{process, str, sync::{Arc, Mutex}};
use structs_enums::TorrentInfo;
use tokio::sync::mpsc;
use uuid::Uuid;

mod bencode;
mod torrent_reader;
mod tracker;
mod peers;
mod structs_enums;

#[tokio::main]
async fn main() {
  const BRAND: &str = "RK";
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

  let mut ti: Arc<TorrentInfo> = Arc::new(TorrentInfo {
    infohash: infohash.clone(),
    announce_list: announce_list,
    num_pieces: num_pieces,
    downloaded: 0,
    size: size,
    pieces: pieces,
    peer_id: session_uuid
  });

  println!("TRACKERS: {:?}", ti.announce_list);

  // temporary arbitrary buffer. should research more and see how big communication actually gets
  let (tracker_tx, tracker_rx) = mpsc::channel(6);
  
  tracker::start_tracker_comm(Arc::clone(&ti), tracker_tx).await;
  
  tokio::spawn(peers::mpsc_p_process(tracker_rx, Arc::clone(&ti)));
    
}