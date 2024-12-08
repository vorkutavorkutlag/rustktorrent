use std::{net::{Ipv4Addr, SocketAddrV4, TcpStream, ToSocketAddrs, UdpSocket}, thread, time::Duration};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use bytes::Bytes;
use tokio::sync::mpsc;
use url::Url;
use rand::Rng;
use crate::bencode;

struct Tracker {
    tracker_url: String,
    port: u16,
    infohash: Vec<u8>,
    size: u64,
    peerid: String,
    downloaded: u64,
    uploaded: u64, 
    interval: u64,
}

fn parse_compact_peers(peers: &Vec<u8>) -> Vec<SocketAddrV4> {
  // 4 bytes for IP + 2 bytes for port
  let mut result = Vec::new();
  let peer_length = 6;

  for chunk in peers.chunks(peer_length) {
    if chunk.len() != peer_length {
      continue
    }
    let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
    let port = u16::from_be_bytes([chunk[4], chunk[5]]);

    let socket_addr = SocketAddrV4::new(ip, port);
    result.push(socket_addr);
  }

  result
}

fn is_port_open(url: &str, port: u16) -> bool {
  let socket_address = format!("{}:{}", url, port);

  match socket_address.to_socket_addrs() {
      Ok(mut addrs) => {
          if let Some(addr) = addrs.next() {
              // Try to connect to the resolved IP address with the specified port
              match TcpStream::connect_timeout(&addr, Duration::from_secs(1)) {
                  Ok(_) => true,  // Connection succeeded, port is open
                  Err(_) => false, // Connection failed, port is closed
              }
          } else {
              false // Failed to resolve any address
          }
      }
      Err(_) => false, // Domain name resolution failed
  }
}

fn parse_tracker_response(body: Bytes, tracker: &mut Tracker) -> Result<Vec<SocketAddrV4>, String> {
  // Given a response body from an HTTP tracker, updates tracker interval and returns peer information
  // Will Err if it cannot parse either interval or adresses

  let decoded = bencode::decode_bencode(&body.to_vec(), &mut 0).map_err(|_| "Failed to decode bencode")?;
  
  if let bencode::Bencode::Dictionary(decoded_dict) = decoded {
    // Extract interval, returning an error if not found
    let interval = decoded_dict
      .get("interval")
      .and_then(|b| if let bencode::Bencode::Integer(i) = b { Some(*i) } else { None })
      .ok_or_else(|| "Missing or invalid 'interval'")?;
      
    tracker.interval = interval as u64;

    // Match and parse peers
    match decoded_dict.get("peers") {
      Some(bencode::Bencode::String(peers)) => {
          Ok(parse_compact_peers(peers))
        }
      Some(bencode::Bencode::List(peers_list)) => {
        let mut peer_ip_port = Vec::new();

        for peer in peers_list {
          if let bencode::Bencode::Dictionary(peer_dict) = peer {
            if let (Some(bencode::Bencode::String(ip)), Some(bencode::Bencode::Integer(port))) = 
              (peer_dict.get("ip"), peer_dict.get("port")) 
                  {
                    if ip.len() == 4 && (0..=u16::MAX as i64).contains(port) {
                      let ip_addr = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
                      peer_ip_port.push(SocketAddrV4::new(ip_addr, *port as u16));
                    } else {
                      return Err("Invalid IP length or port".to_string());
                    }
                  }
               }
            }
        Ok(peer_ip_port)
        }
        _ => Err("Missing or invalid 'peers' data".to_string()),
    }
  } else {
      Err("Decoded data is not a dictionary".to_string())
  }
}


async fn http_comm(mut tracker: Tracker, tx: mpsc::Sender<Vec<SocketAddrV4>>) -> () {
 // First, find the port on which the tracker is listening, if it is not given.
 // By default, port == 0, unless we successfully parsed it from the url.
  if tracker.port == 0 {
    let mut potential_ports = (6881..6889).collect::<Vec<u16>>();
    potential_ports.extend([6969, 80]);
    for port in potential_ports {
      if is_port_open(&tracker.tracker_url, port) {
        tracker.port = port;
        break 
        }
      // if port is still zero, none of the default ports are open and we don't have a destined port
      return
      }
    }

  // initialize reqwest client
  let client = reqwest::Client::new();
  
  // URL-encode the `info_hash`
  let info_hash_encoded = percent_encode(&tracker.infohash, NON_ALPHANUMERIC).to_string();
  let mut main_url = format!("{}?info_hash={}", tracker.tracker_url, info_hash_encoded);
  main_url.push_str(&format!(
      "&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event=started&compact=1",
      tracker.peerid,
      tracker.port,
      tracker.uploaded,
      tracker.downloaded,
      tracker.size - tracker.downloaded,
  ));

  loop {
    let res = match client.get(&main_url).header("User-Agent", reqwest::header::USER_AGENT).send().await {
      Ok(response) => response,
      Err(_) => {tokio::time::sleep(Duration::from_secs(tracker.interval)).await; continue},
    };
    
    let body = match res.bytes().await {
        Ok(bod) => bod,
        Err(_) => {tokio::time::sleep(Duration::from_secs(tracker.interval)).await; continue},
    };

    match parse_tracker_response(body, &mut tracker) {
      Ok(peer_addresses) => {println!("Sending peers... interval:{}", tracker.interval); tx.send(peer_addresses).await.unwrap()},
      Err(_) => {tokio::time::sleep(Duration::from_secs(tracker.interval)).await; continue},
    }

    tokio::time::sleep(Duration::from_secs(tracker.interval)).await;
  }
}


fn udp_connection_request(socket: &mut UdpSocket) -> Option<(i64, u16)> {
  const MAGIC_CONSTANT: i64 = 0x41727101980; // Magic constant for protocol
  const CONNECT_ACTION: i32 = 0; 
  const RESPONSE_LENGTH: usize = 16;

  let transaction_id: u16 = rand::thread_rng().gen_range(0..=65535);
  
  let mut packet: Vec<u8> = vec![];
  packet.extend(&MAGIC_CONSTANT.to_be_bytes());
  packet.extend(&CONNECT_ACTION.to_be_bytes());
  packet.extend(&transaction_id.to_be_bytes());

  socket.send(&packet).unwrap();

  let mut response = [0u8; RESPONSE_LENGTH];
  let (size, _) = socket.recv_from(&mut response).unwrap();

  if size != RESPONSE_LENGTH {
    return None;
  }
  
  // parse connection response
  let res_action = i32::from_be_bytes(response[0..4].try_into().unwrap());
  let res_transaction = u16::from_be_bytes(response[4..8].try_into().unwrap());
  let connection_id = i64::from_be_bytes(response[8..16].try_into().unwrap());
  
  // validation
  if res_action != CONNECT_ACTION || res_transaction != transaction_id {
    return None
  }

  // return connection ID and transaction ID
  Some((connection_id, transaction_id))

}

async fn udp_comm(mut tracker: Tracker, tx: mpsc::Sender<Vec<SocketAddrV4>>) -> () {
  let url = match url::Url::parse(&tracker.tracker_url) {
    Ok(url) => url,
    Err(_) => return    // unable to resolve dns - no point in trying more
  };
  
  // tracker.port == 0 means we did not find a default port, so we need to pass None
  let addrs: Vec<_> = match url.socket_addrs(||  if tracker.port > 0 {Some(tracker.port)} else {None}) {
    Ok(addrs) => addrs
        .into_iter()
        .filter(|addr| addr.is_ipv4()) 
        .collect::<Vec<_>>(),
    Err(e) => {
        eprintln!(
            "Failed to resolve tracker addresses for {}: {}",
            tracker.tracker_url, e
        );
        return;
    }
  };

  if addrs.is_empty() {
    return;
  }

  println!("{:#?}", addrs);
  let mut socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();          // bind socket
  socket.connect(&*addrs).unwrap();                                                          // actually connect it to server
  
  // loop until we get a good response
  let (connection_id, transaction_id) = loop {
    if let Some(vals) = udp_connection_request(&mut socket) {
      break vals;
    } else {
      tokio::time::sleep(Duration::from_secs(tracker.interval)).await;
    }
  };
  
  
  const CONNECT_ACTION: i32 = 1; 
  const DEFAULT_IP: u8 = 0;
  const DEFAULT_NUMWANT: i8 = -1;
  let EVENT: u8 = 2;    // temporary, should be dynamically decided as started, completed, stopped (2, 1, 3)

  loop {
    let key: u16 = rand::thread_rng().gen_range(0..=65535);
    let left: u64 = tracker.downloaded - tracker.size;
    let mut packet: Vec<u8> = vec![];
    packet.extend(&connection_id.to_be_bytes());
    packet.extend(&CONNECT_ACTION.to_be_bytes());
    packet.extend(&transaction_id.to_be_bytes());
    packet.extend(&tracker.infohash);
    packet.extend(tracker.peerid.as_bytes());
    packet.extend(&tracker.downloaded.to_be_bytes());
    packet.extend(&left.to_be_bytes());
    packet.extend(&tracker.uploaded.to_be_bytes());
    packet.extend(&connection_id.to_be_bytes());
    packet.extend(&EVENT.to_be_bytes());
    packet.extend(&DEFAULT_IP.to_be_bytes());
    packet.extend(&key.to_be_bytes());
    packet.extend(&DEFAULT_NUMWANT.to_be_bytes());
    packet.extend(&tracker.port.to_be_bytes());

    socket.send(&packet).unwrap();

    // initial response size is constant, 20 bytes
    let mut response_metadata: Vec<u8> = vec![0u8; 20]; 
    let (size, _) = socket.recv_from(&mut response_metadata).unwrap();

    if size < 20 {
      tokio::time::sleep(Duration::from_secs(tracker.interval)).await;
      continue;
    }

    let action = i32::from_be_bytes(response_metadata[0..4].try_into().unwrap());
    let res_transaction = u16::from_be_bytes(response_metadata[4..8].try_into().unwrap());

    if action != CONNECT_ACTION || res_transaction != transaction_id {
      tokio::time::sleep(Duration::from_secs(tracker.interval)).await;
      continue;
    }

    let interval = u64::from_be_bytes(response_metadata[8..12].try_into().unwrap());
    let leechers = u32::from_be_bytes(response_metadata[12..16].try_into().unwrap()); 
    let seeders = u32::from_be_bytes(response_metadata[16..20].try_into().unwrap());  


    // each ip address is composed of 6 bytes. meaning the sum size is the number of all peers times 6
    let peer_data_len: usize= ((leechers + seeders) * 6) as usize;
    let mut response: Vec<u8> = vec![0u8; peer_data_len]; 
    let _ = socket.recv_from(&mut response).unwrap();

    let mut peers: Vec<SocketAddrV4> = vec![];
    let mut index = 0;
    while index + 6 <= size {
        let ip = Ipv4Addr::new(response[index], response[index + 1], response[index + 2], response[index + 3]);
        let port = u16::from_be_bytes([response[index + 4], response[index + 5]]);
        peers.push(SocketAddrV4::new(ip, port));
        index += 6;
    }

    tx.send(peers).await.unwrap();
    
    tracker.interval = interval;
    tokio::time::sleep(Duration::from_secs(tracker.interval)).await;

  }
}

pub async fn start_tracker_comm(infohash: Vec<u8>, mut announce_list: Vec<String>, size: u64, session_uuid: String, downloaded: u64, tx: mpsc::Sender<Vec<SocketAddrV4>>) {
    let mut udp_trackers = Vec::new();
    let mut http_trackers = Vec::new();
    
    while let Some(tracker_url) = announce_list.pop() {
      let parsed_url = match Url::parse(&tracker_url) {
        Ok(parsed) => parsed,
        Err(_) => {eprintln!("Invalid announce url - Ignoring"); continue;}
      };
      
      let port: u16 = parsed_url.port().unwrap_or(0);

      let mut tracker: Tracker = Tracker 
       {tracker_url: tracker_url,
        port: port,
        infohash: infohash.clone(),
        size: size,
        peerid: session_uuid.clone(),
        downloaded: downloaded,
        interval: 60, uploaded: 0};
      
      let tx_clone: mpsc::Sender<Vec<SocketAddrV4>> = tx.clone();
      match parsed_url.scheme() {
        "http" | "https" => http_trackers.push(tokio::spawn(http_comm(tracker, tx_clone))),
        "udp" => udp_trackers.push(tokio::spawn(udp_comm(tracker, tx_clone))),
        _ => continue
      }
  }
}