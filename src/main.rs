use std::process;
mod bencode;

fn main() {
    let file_path: &str = "sonic.torrent";
    match bencode::read_torrent(file_path) {
        Ok(()) => (),
        Err(e) => {println!("{}", e); process::exit(0)}
    }
}
