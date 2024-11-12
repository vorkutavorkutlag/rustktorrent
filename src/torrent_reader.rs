use bencode::Bencode;
mod bencode;

fn prase_torrent(torrent_hashmap: &HashMap<String, Bencode>) {
    // hashed info == sha1 function on bencoded info key
    // announce list == announce key and announce-list key
    // piece_length == piece length key of the info key
    // size == length key of the info key or sum of all length keys in the files key in the info key
    // num_pieces == ceiiling of size / piece_length
}