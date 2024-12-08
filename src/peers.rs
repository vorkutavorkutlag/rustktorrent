use std::{net::{TcpStream}};

async fn do_handshake(t_peer: SocketAddrV4, infohash: &[u8], peer_id: &str) -> Result<TcpStream, String> {
  let mut socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
  socket.connect(t_peer)?;
  
  let protocol: [u8] = "Bittorrent Protocol".to_be_bytes();
  let p_len: u8 = protocol.len();
  let reserved: [u8] = [0u8; 8];
  let pid_bytes: [u8] = peer_id.to_bytes();
  
  let mut packet: Vec<u8> = vec![];

  packet.extend(p_len);
  packet.extend(protocol);
  packet.extend(&reserved);
  packet.extend(&info_hash);
  packet.extend(&pid_bytes);

  socket.send(&packet)?;

  let mut response_data: Vec<u8> = vec![0u8; 68]; 
  let (size, _) = socket.recv_from(&mut response_data)?;

  let valid_response: bool = True;
  
  valid_response = valid_response && size == 68;
  valid_response = valid_response && response_data[0..2] == p_len;
  valid_response = valid_response && response_data[2..p_len] == protocol;
  valid_response = valid_response && response_data[p_len..p_len+8] == reserved;
  valid_response = valid_response && response_data[p_len+8..p_len+28] == info_hash;
  
  if valid_response {
    return Ok(socket);
  }

  return Err("Invalid response");



}