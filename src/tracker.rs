use std::thread;

struct Tracker {
    client_ip: String,
    tracker_url: String,
    infohash: Vec<u8>,
    size: i64,
    peerid: String,
    downloaded: i64,
    uploaded: i64, 
    interval: i64,
}

async fn http_comm(tracker: Tracker) -> String {
    // Set up reqwest client
    let client = reqwest::Client::new();
    
    // URL-encode the `info_hash` and `peer_id`
    let info_hash_encoded: &String = &tracker.infohash.iter().map(|byte| format!("%{:02X}", byte)).collect::<String>();


    for port in 6881..6889 { // bittorrent http protocol moves between these ports, should try them all
        // let mut url = reqwest::Url::parse(&tracker.tracker_url).expect("Invalid tracker URL");
        // url.query_pairs_mut()
        //     .append_pair("info_hash", info_hash_encoded)
        //     .append_pair("peer_id", &tracker.peerid)
        //     .append_pair("port", &port.to_string())
        //     .append_pair("uploaded", &tracker.uploaded.to_string())
        //     .append_pair("downloaded", &tracker.downloaded.to_string())
        //     .append_pair("left", &(tracker.size - tracker.downloaded).to_string())
        //     .append_pair("event", "started")
        //     .append_pair("compact", "1");

        let mut url = format!("{}?info_hash={}", tracker.tracker_url, info_hash_encoded);
        url.push_str(&format!(
            "&peer_id={}&ip={}&port={}&uploaded={}&downloaded={}&left={}&event=started&compact=1",
            tracker.peerid,
            tracker.client_ip,
            port,
            tracker.uploaded,
            tracker.downloaded,
            tracker.size - tracker.downloaded,
        ));


        // return format!("{:#?}", url.to_string());
        
        let res = match client.get(url).send().await {
            Ok(response) => response,
            Err(_e) => {
                // eprintln!("Bad response, {:#?}", e);
                continue;
            }
        };
        
        let body = match res.text().await {
            Ok(bod) => bod,
            Err(e) => {
                eprintln!("Bad body: {:#?}", e);
                continue;
            }
        };

        return body;
    }
    String::new()
}

async fn udp_comm(tracker: Tracker) {
    // socket comm
}

pub async fn start_tracker_comm(infohash: Vec<u8>, announce_list: Vec<String>, size: i64, session_uuid: String, downloaded: i64, ip_addr: String) {
    println!("Starting tracker comm..! {}", format!("{:#?}", announce_list));
    let mut udp_trackers = Vec::new();
    let mut http_trackers = Vec::new();
    
    for tracker_url in announce_list {
        let tracker: Tracker = Tracker {client_ip: ip_addr.clone(),
                                        tracker_url: tracker_url.clone(),
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