use std::{collections::HashMap, thread};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use crate::bencode;

struct Tracker {
    tracker_url: String,
    infohash: Vec<u8>,
    size: i64,
    peerid: String,
    downloaded: i64,
    uploaded: i64, 
    interval: i64,
}

async fn http_comm(mut tracker: Tracker) -> String {
    // Set up reqwest client
    let client = reqwest::Client::new();
    
    // URL-encode the `info_hash`
    let info_hash_encoded = percent_encode(&tracker.infohash, NON_ALPHANUMERIC).to_string();

    for port in 6881..6889 { // bittorrent http protocol moves between these ports, should try them all
        let mut url = format!("{}?info_hash={}", tracker.tracker_url, info_hash_encoded);
        url.push_str(&format!(
            "&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event=started&compact=0",
            tracker.peerid,
            port,
            tracker.uploaded,
            tracker.downloaded,
            tracker.size - tracker.downloaded,
        ));

        
        let res = match client.get(url).header("User-Agent", reqwest::header::USER_AGENT).send().await {
            Ok(response) => response,
            Err(_e) => continue
        };
        
        let body = match res.bytes().await {
            Ok(bod) => bod,
            Err(e) => continue
        };

        if let bencode::Bencode::Dictionary(decoded) = bencode::decode_bencode(&body.to_vec(), & mut 0).unwrap() {
            if let Some(bencode::Bencode::Integer(interval)) = decoded.get("interval") {
                tracker.interval = *interval;
            }
            
            if let Some(bencode::Bencode::String(peers)) = decoded.get("peers") {            // compact version
                return String::from_utf8(peers.clone()).unwrap()
            }

            else if let Some(bencode::Bencode::List(peers)) = decoded.get("peers") {    // Non-compact version
                let mut peer_ip_port: HashMap<String, i64> = HashMap::new();
                for peer in peers {
                    if let bencode::Bencode::Dictionary(peer_dict) = peer {
                        if let Some(bencode::Bencode::String(ip)) = peer_dict.get("ip") {
                            if let Some(bencode::Bencode::Integer(port)) = peer_dict.get("port") {
                                let ip = String::from_utf8(ip.to_vec()).unwrap();
                                peer_ip_port.insert(ip, *port);
                            }
                        }
                    }
                }
            return format!("{:#?}", peer_ip_port);
            }
        }
        // cursed little shit.
    }
    String::new()
}

async fn udp_comm(tracker: Tracker) {
    // socket comm
}

pub async fn start_tracker_comm(infohash: Vec<u8>, announce_list: Vec<String>, size: i64, session_uuid: String, downloaded: i64) {
    println!("Starting tracker comm..! {}", format!("{:#?}", announce_list));
    let mut udp_trackers = Vec::new();
    let mut http_trackers = Vec::new();
    
    for tracker_url in announce_list {
        let mut tracker: Tracker = Tracker {tracker_url: tracker_url.clone(),
                                        infohash: infohash.clone(),
                                        size: size,
                                        peerid: session_uuid.clone(),
                                        downloaded: downloaded,
                                        interval: 0,
                                        uploaded: 0};


        if tracker_url.starts_with("udp") {
            udp_trackers.push(thread::spawn(move || {udp_comm(tracker)}));
        } else if tracker_url.starts_with("http") {
            http_trackers.push(thread::spawn(move || {http_comm(tracker)}));
        } else {eprintln!("Invalid announce url - Ignoring"); continue;}
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