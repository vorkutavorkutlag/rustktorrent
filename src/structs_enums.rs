use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TorrentInfo {
  pub infohash: Vec<u8>,
  pub announce_list: Vec<String>,
  pub num_pieces: u64,
  pub downloaded: u64,
  pub size: u64,
  pub pieces: Vec<u8>,
  pub peer_id: String,
}

pub struct Tracker {
  pub tracker_url: String,
  pub port: u16,
  pub infohash: Vec<u8>,
  pub size: u64,
  pub peerid: String,
  pub downloaded: u64,
  pub uploaded: u64, 
  pub interval: u64,
}

#[derive(Debug, Clone)]
pub enum Bencode {
  Integer(i64),
  String(Vec<u8>),
  List(Vec<Bencode>),
  Dictionary(HashMap<String, Bencode>),
}

pub enum BittorrentConstants {
  Choke = 0,
  Unchoke = 1,
  Interested = 2,
  NotInterested = 3,
  Have = 4,
  Bitfield = 5,
  Request = 6,
  Piece = 7,
  Cancel = 8,
  Port = 9,
  BlockLength = 16384 // 2^14
}

impl TryFrom<u8> for BittorrentConstants {
  type Error = &'static str;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
      match value {
          0 => Ok(BittorrentConstants::Choke),
          1 => Ok(BittorrentConstants::Unchoke),
          2 => Ok(BittorrentConstants::Interested),
          3 => Ok(BittorrentConstants::NotInterested),
          4 => Ok(BittorrentConstants::Have),
          5 => Ok(BittorrentConstants::Bitfield),
          6 => Ok(BittorrentConstants::Request),
          7 => Ok(BittorrentConstants::Piece),
          8 => Ok(BittorrentConstants::Cancel),
          9 => Ok(BittorrentConstants::Port),
          _ => Err("Invalid value for BittorrentConstants"),
      }
  }
}