struct Tracker {
    infohash: String,
    size: i64,
    peerid: Vec<u8>,
    downloaded: i64, 
    interval: i32,
}

fn http_comm() {
    // http requests
}

fn udp_comm() {
    // socket comm
}

fn start_tracker_comm(infohash: String, announce_list: Vec<String>, size: i64, session_uuid: Vec<u8>, downloaded: i64) {
    
    for tracker_url in announce_list {
        let tracker: Tracker = Tracker {infohash: infohash.clone(),
                                        size: size,
                                        peerid: session_uuid.clone(),
                                        downloaded: downloaded,
                                        interval: 0};
        
        if tracker_url.starts_with("http") {

        } else if tracker_url.starts_with("udp") {

        } else {continue}
    }
}