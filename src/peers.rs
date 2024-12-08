use std::{net::{TcpStream, SocketAddrV4}, io::prelude::*};

async fn do_handshake(t_peer: SocketAddrV4, infohash: &[u8], peer_id: &str) -> Result<TcpStream, String> {
  let mut socket = TcpStream::connect(t_peer).map_err(|e| e.to_string())?;
  
  let protocol: &[u8] = "Bittorrent Protocol".as_bytes();
  let p_len: usize = protocol.len();
  let reserved = [0u8; 8];
  let pid_bytes: &[u8] = peer_id.as_bytes();
  
  let mut packet: Vec<u8> = vec![];

  packet.extend(&p_len.to_be_bytes());
  packet.extend(protocol);
  packet.extend(&reserved);
  packet.extend(infohash);
  packet.extend(pid_bytes);

  socket.write_all(&packet).map_err(|e| e.to_string())?;

  let mut response_data: Vec<u8> = vec![0u8; 68]; 
  let size = socket.read(&mut response_data).map_err(|e| e.to_string())?;

  if size != 68 {
    return Err("Invalid response".to_string());
  }

  let mut valid_response: bool = true;
  valid_response = valid_response && response_data[0..2] == p_len.to_be_bytes();
  valid_response = valid_response && response_data[2..p_len] == *protocol;
  valid_response = valid_response && response_data[p_len..p_len+8] == reserved;
  valid_response = valid_response && response_data[p_len+8..p_len+28] == *infohash;
  
  if valid_response {
    return Ok(socket);
  }

  return Err("Invalid response".to_string());
}