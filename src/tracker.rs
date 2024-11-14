use std::thread;

struct Tracker {
    tracker_url: String,
    infohash: String,
    size: i64,
    peerid: Vec<u8>,
    downloaded: i64, 
    interval: i32,
}

fn http_comm(tracker: Tracker) {
    // http requests
}

fn udp_comm(tracker: Tracker) {
    // socket comm
}

fn start_tracker_comm(infohash: String, announce_list: Vec<String>, size: i64, session_uuid: Vec<u8>, downloaded: i64) {
    
    let mut trackers: Vec<thread::JoinHandle<()>> = vec![];

    for tracker_url in announce_list {
        let tracker: Tracker = Tracker {tracker_url: tracker_url.clone(),
                                        infohash: infohash.clone(),
                                        size: size,
                                        peerid: session_uuid.clone(),
                                        downloaded: downloaded,
                                        interval: 0};
        
        if tracker_url.starts_with("http") {
            trackers.push(thread::spawn(move || {http_comm(tracker)}));
        } else if tracker_url.starts_with("udp") {
            trackers.push(thread::spawn(move || {udp_comm(tracker)}));
        } else {eprintln!("Invalid announce url - Ignoring"); continue;}
    }
}