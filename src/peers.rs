use std::{net::{TcpStream, SocketAddrV4}, error::Error, sync::{Arc, Mutex}, io::prelude::*};

use tokio::sync::mpsc::Receiver;

use crate::structs_enums;
use structs_enums::{TorrentInfo, BittorrentConstants};

async fn do_handshake(t_peer: SocketAddrV4, ti: &'static TorrentInfo) -> Result<TcpStream, Box <dyn Error>> {
  let mut socket = TcpStream::connect(t_peer)?;
  
  let protocol: &[u8] = "Bittorrent Protocol".as_bytes();
  let p_len: usize = protocol.len();
  let reserved = [0u8; 8];
  let pid_bytes: &[u8] = ti.peer_id.as_bytes();
  
  let mut packet: Vec<u8> = vec![];

  packet.extend(&p_len.to_be_bytes());
  packet.extend(protocol);
  packet.extend(&reserved);
  packet.extend(&ti.infohash);
  packet.extend(pid_bytes);

  socket.write_all(&packet)?;
  drop(packet);

  let mut response_data: Vec<u8> = vec![0u8; 68]; 
  let size = socket.read(&mut response_data).map_err(|e| e.to_string())?;

  if size != 68 {
    return Err("Invalid response".into());
  }

  let mut valid_response: bool = true;
  valid_response = valid_response && response_data[0..2] == p_len.to_be_bytes();
  valid_response = valid_response && response_data[2..p_len] == *protocol;
  valid_response = valid_response && response_data[p_len..p_len+8] == reserved;
  valid_response = valid_response && response_data[p_len+8..p_len+28] == *ti.infohash;
  
  if valid_response {
    return Ok(socket);
  }

  Err("Invalid response".into())
}

async fn interested_msg(mut t_peer: &TcpStream, ti: &Arc<TorrentInfo>) -> Result<(), String> {
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

  Ok(())
}

async fn p_process(t_peer: SocketAddrV4, ti: Arc<TorrentInfo>) {
  todo!();
}

pub async fn mpsc_p_process(mut tracker_rx: Receiver<Vec<SocketAddrV4>>, ti: Arc<TorrentInfo>) {
  let mut peer_threads = vec![];
  while let Some(peers) = tracker_rx.recv().await {
    for peer in peers {
      peer_threads.push(tokio::spawn(p_process(peer, Arc::clone(&ti))));
    }
  }
}