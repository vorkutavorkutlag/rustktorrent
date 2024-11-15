use std::thread;
use reqwest::header::USER_AGENT;
use tokio;

struct Tracker {
    tracker_url: String,
    infohash: Vec<u8>,
    size: i64,
    peerid: Vec<u8>,
    downloaded: i64,
    uploaded: i64, 
    interval: i64,
}

async fn http_comm(tracker: Tracker) -> String {
    let client = reqwest::Client::new();

    for port in 6881..6889 { // bittorrent http protocol moves between these ports, should try them all
        let res = match client
        .get(tracker.tracker_url.clone())
        .header("info_hash", tracker.infohash.clone())
        .header("peer_id", tracker.peerid.clone())
        .header("port", port)
        .header("uploaded", tracker.uploaded)
        .header("downloaded", tracker.downloaded)
        .header("left", tracker.size - tracker.downloaded)
        .header("event", "started")
        .header("compact", "1")
        .send()
        .await {
            Ok(response) => response,
            Err(_) => continue
        };

        let body = match res.text().await {
            Ok(bod) => bod, 
            Err(_)=>continue};

        return body;
    }
    String::new()
}

async fn udp_comm(tracker: Tracker) {
    // socket comm
}

pub async fn start_tracker_comm(infohash: Vec<u8>, announce_list: Vec<String>, size: i64, session_uuid: Vec<u8>, downloaded: i64) {
    println!("Starting tracker comm..! {}", format!("{:#?}", announce_list));
    let mut udp_trackers = Vec::new();
    let mut http_trackers = Vec::new();
    
    for tracker_url in announce_list {
        let tracker: Tracker = Tracker {tracker_url: tracker_url.clone(),
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
            Ok(_) => println!("ok"),
            Err(_) => println!("err")
        }
    }

}