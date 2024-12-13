use crate::bencode;
use std::collections::HashSet;
use sha1::{Sha1, Digest};

use crate::structs_enums;
use structs_enums::Bencode;

pub fn parse_torrent(filename: &str) -> Result<(Vec<u8>, HashSet<String>, u64, u64, u64, Vec<u8>), String> {
  // Decode the torrent file into a HashMap
  let torrent = match bencode::read_torrent_file(filename) {
    Ok(bytes) => {
      let mut index = 0;
      match bencode::decode_bencode(&bytes, &mut index) {
        Ok(Bencode::Dictionary(map)) => map,
        _ => return Err("Failed to decode torrent into a dictionary".to_string()),
        }
      },
    Err(e) => return Err(format!("Error reading file: {}", e)),
  };

    // Extract and hash the 'info' key
  let info_key = torrent.get("info").ok_or("Missing 'info' key")?;
  let info_bencoded = bencode::encode_bencode(info_key);
  
  let mut hasher = Sha1::new();
  hasher.update(&info_bencoded);
  let hash_result = hasher.finalize();
  let hashed_info= hash_result.to_vec();

    // Get the announce list (single announce and 'announce-list')
  let mut announce_list = HashSet::new();
  if let Some(Bencode::String(announce)) = torrent.get("announce") {
      announce_list.insert(String::from_utf8(announce.clone()).map_err(|_| "Invalid UTF-8 in 'announce'")?);
  }
  if let Some(Bencode::List(ann_list)) = torrent.get("announce-list") {
    for entry in ann_list {
      if let Bencode::List(url_list) = entry {
        for url in url_list {
          if let Bencode::String(url_bytes) = url {
            announce_list.insert(String::from_utf8(url_bytes.clone()).map_err(|_| "Invalid UTF-8 in 'announce-list'")?);
                  }
              }
          }
      }
  }


  // Get piece length, size and pieces hash
  let info_map = match info_key {
    Bencode::Dictionary(map) => map,
    _ => return Err("The 'info' key is not a dictionary".to_string()),
  };
  
  let piece_length: u64 = match info_map.get("piece length") {
    Some(Bencode::Integer(len)) => *len as u64,
    _ => return Err("Missing or invalid 'piece length'".to_string()),
  };

  let pieces: Vec<u8> = match info_map.get("pieces") {
    Some(Bencode::String(pieces)) => pieces.to_vec(),
    _ => return Err("Missing or invalid 'pieces'".to_string()),
  };

  // Calculate size
  let size: u64 = if let Some(Bencode::Integer(len)) = info_map.get("length") {
      *len as u64
  
  } else if let Some(Bencode::List(files)) = info_map.get("files") {
    let mut total_size: u64 = 0;
    for file in files {
      if let Bencode::Dictionary(file_map) = file {
        if let Some(Bencode::Integer(len)) = file_map.get("length") {
          total_size += *len as u64;
              }
          }
      }
      total_size
  } else {
      return Err("Missing 'length' or 'files' in 'info'".to_string());
  };

  // Calculate num_pieces
  let num_pieces = (size as f64 / piece_length as f64).ceil() as u64;
  
  Ok((hashed_info, announce_list, piece_length, size, num_pieces, pieces))
}