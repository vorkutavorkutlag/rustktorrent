use std::{net::{Ipv4Addr, SocketAddrV4, TcpStream, ToSocketAddrs}, time::Duration, thread};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use bytes::Bytes;
use url::Url;
use crate::bencode;

struct Tracker {
    tracker_url: String,
    port: u16,
    infohash: Vec<u8>,
    size: i64,
    peerid: String,
    downloaded: i64,
    uploaded: i64, 
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

fn parse_http(body: Bytes, tracker: &mut Tracker) -> Result<Vec<SocketAddrV4>, String> {
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


async fn http_comm(mut tracker: Tracker) -> () {
 // First, find the port on which the tracker is listening, if it is not given.
 // By default, port == 0, unless we successfully parsed it from the url.
  if tracker.port == 0 {
    let mut potential_ports = (6881..6889).collect::<Vec<u16>>();
    potential_ports.push(6969);
    for port in potential_ports {
      if is_port_open(&tracker.tracker_url, port) {
        tracker.port = port;
        }
      }
    }
  
  if tracker.port == 0 {
    // None of the default ports are open and we don't have a destined port
    return
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
      let res = match client.get(main_url).header("User-Agent", reqwest::header::USER_AGENT).send().await {
        Ok(response) => response,
        Err(_) => {thread::sleep(Duration::from_secs(tracker.interval)); continue},
      };
    
      let body = match res.bytes().await {
          Ok(bod) => bod,
          Err(_) => {thread::sleep(Duration::from_secs(tracker.interval)); continue},
      };

      match parse_http(body, &mut tracker) {
        Ok(peer_addresses) => todo!("Send peer adresses to arc"),
        Err(_) => {thread::sleep(Duration::from_secs(tracker.interval)); continue},
      }
  }
}

async fn udp_comm(tracker: Tracker) {
    // socket comm
}

pub async fn start_tracker_comm(infohash: Vec<u8>, announce_list: Vec<String>, size: i64, session_uuid: String, downloaded: i64) {
    println!("Starting tracker comm..! {}", format!("{:#?}", announce_list));
    let mut udp_trackers = Vec::new();
    let mut http_trackers = Vec::new();
    
    for tracker_url in announce_list {
      let parsed_url = match Url::parse(&tracker_url) {
        Ok(parsed) => parsed,
        Err(_) => {eprintln!("Invalid announce url - Ignoring"); continue;}
      };
      
      let port = parsed_url.port().unwrap_or(0);

      let mut tracker: Tracker = Tracker 
       {tracker_url: tracker_url.clone(),
        port: port,
        infohash: infohash.clone(),
        size: size,
        peerid: session_uuid.clone(),
        downloaded: downloaded,
        interval: 0, uploaded: 0};

      match parsed_url.scheme() {
        "http" => http_trackers.push(thread::spawn(move || {http_comm(tracker)})),
        "udp" => udp_trackers.push(thread::spawn(move || {udp_comm(tracker)})),
        _ => {eprintln!("Invalid announce url - Ignoring"); continue;}
      }
  }

  for tracker in http_trackers {
    match tracker.join() { 
      Ok(ok) => println!("{:#?}", ok.await),
      Err(_) => println!("err")
      }
  }

    
  for tracker in udp_trackers {
    match tracker.join() { 
      Ok(ok) =>  println!("{:#?}", ok.await),
      Err(_) => println!("err")
      }
  }

}