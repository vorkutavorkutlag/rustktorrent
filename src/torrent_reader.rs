use crate::bencode;
use sha1::{Sha1, Digest};


pub fn parse_torrent(filename: &str) -> Result<(Vec<u8>, Vec<String>, u64, u64, u64, Vec<u8>), String> {
  // Decode the torrent file into a HashMap
  let torrent = match bencode::read_torrent_file(filename) {
    Ok(bytes) => {
      let mut index = 0;
      match bencode::decode_bencode(&bytes, &mut index) {
        Ok(bencode::Bencode::Dictionary(map)) => map,
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
  println!("{:x}", hash_result);
  let hashed_info= hash_result.to_vec();

    // Get the announce list (single announce and 'announce-list')
  let mut announce_list = Vec::new();
  if let Some(bencode::Bencode::String(announce)) = torrent.get("announce") {
      announce_list.push(String::from_utf8(announce.clone()).map_err(|_| "Invalid UTF-8 in 'announce'")?);
  }
  if let Some(bencode::Bencode::List(ann_list)) = torrent.get("announce-list") {
    for entry in ann_list {
      if let bencode::Bencode::List(url_list) = entry {
        for url in url_list {
          if let bencode::Bencode::String(url_bytes) = url {
            announce_list.push(String::from_utf8(url_bytes.clone()).map_err(|_| "Invalid UTF-8 in 'announce-list'")?);
                  }
              }
          }
      }
  }


  // Get piece length, size and pieces hash
  let info_map = match info_key {
    bencode::Bencode::Dictionary(map) => map,
    _ => return Err("The 'info' key is not a dictionary".to_string()),
  };
  
  let piece_length: u64 = match info_map.get("piece length") {
    Some(bencode::Bencode::Integer(len)) => *len as u64,
    _ => return Err("Missing or invalid 'piece length'".to_string()),
  };

  let pieces: Vec<u8> = match info_map.get("pieces") {
    Some(bencode::Bencode::String(pieces)) => pieces.to_vec(),
    _ => return Err("Missing or invalid 'pieces'".to_string()),
  };

  // Calculate size
  let size: u64 = if let Some(bencode::Bencode::Integer(len)) = info_map.get("length") {
      *len as u64
  
  } else if let Some(bencode::Bencode::List(files)) = info_map.get("files") {
    let mut total_size: u64 = 0;
    for file in files {
      if let bencode::Bencode::Dictionary(file_map) = file {
        if let Some(bencode::Bencode::Integer(len)) = file_map.get("length") {
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