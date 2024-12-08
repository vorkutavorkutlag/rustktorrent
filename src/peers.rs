use std::{net::{TcpStream, SocketAddrV4}, io::prelude::*};

use sha1::digest::typenum::Bit;

enum BittorrentConstants {
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

struct TorrentInfo {
  infohash: &'static [u8],
  num_pieces: u64,
  pieces: &'static [u8],
  peer_id: &'static str,
}

async fn do_handshake(t_peer: SocketAddrV4, ti: &'static TorrentInfo) -> Result<TcpStream, String> {
  let mut socket = TcpStream::connect(t_peer).map_err(|e| e.to_string())?;
  
  let protocol: &[u8] = "Bittorrent Protocol".as_bytes();
  let p_len: usize = protocol.len();
  let reserved = [0u8; 8];
  let pid_bytes: &[u8] = ti.peer_id.as_bytes();
  
  let mut packet: Vec<u8> = vec![];

  packet.extend(&p_len.to_be_bytes());
  packet.extend(protocol);
  packet.extend(&reserved);
  packet.extend(ti.infohash);
  packet.extend(pid_bytes);

  socket.write_all(&packet).map_err(|e| e.to_string())?;
  drop(packet);

  let mut response_data: Vec<u8> = vec![0u8; 68]; 
  let size = socket.read(&mut response_data).map_err(|e| e.to_string())?;

  if size != 68 {
    return Err("Invalid response".to_string());
  }

  let mut valid_response: bool = true;
  valid_response = valid_response && response_data[0..2] == p_len.to_be_bytes();
  valid_response = valid_response && response_data[2..p_len] == *protocol;
  valid_response = valid_response && response_data[p_len..p_len+8] == reserved;
  valid_response = valid_response && response_data[p_len+8..p_len+28] == *ti.infohash;
  
  if valid_response {
    return Ok(socket);
  }

  return Err("Invalid response".to_string());
}

async fn interested_msg(mut t_peer: &TcpStream, ti: TorrentInfo) -> Result<(), String> {
  let length_prefix: u8 = 1;
  let msg_id: u8 = BittorrentConstants::Interested as u8;

  let mut packet: Vec<u8> = vec![];

  packet.extend(&length_prefix.to_be_bytes());
  packet.extend(&msg_id.to_be_bytes());

  t_peer.write(&packet).map_err(|e| e.to_string())?;
  drop(packet);

  // obligatory 2 bytes of len prefix and msg id
  // potentially a bitfield message, so we allocate space for it
  let mut response_data: Vec<u8> = vec![0u8; 2 + ti.num_pieces as usize]; 
  t_peer.read(&mut response_data).map_err(|e| e.to_string())?;

  match BittorrentConstants::try_from(response_data[1])? {
    BittorrentConstants::Choke => todo!(),
    BittorrentConstants::Unchoke => todo!(),
    BittorrentConstants::Bitfield => todo!(),
    _ => todo!()
  }

  return Ok(());
}