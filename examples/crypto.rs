extern crate snail;
extern crate env_logger;
extern crate snow;

use snail::vpn::crypto::{self, Handshake};

fn main() {
    env_logger::init();

    let (server_pubkey, server_privkey) = crypto::gen_key().unwrap();
    let (client_pubkey, client_privkey) = crypto::gen_key().unwrap();

    let mut initiator = Handshake::initiator(&server_pubkey, &client_privkey).unwrap();
    let mut responder = Handshake::responder(&server_privkey).unwrap();

    // step 1
    let msg = initiator.take().unwrap();
    responder.insert(&msg).unwrap();

    // step 2
    let msg = responder.take().unwrap();
    initiator.insert(&msg).unwrap();

    // step 3
    let msg = initiator.take().unwrap();
    responder.insert(&msg).unwrap();

    let mut initiator = initiator.transport().expect("initiator.transport");
    let mut responder = responder.transport().expect("responder.transport");

    let remote_pubkey = responder.remote_pubkey().unwrap();
    if client_pubkey == remote_pubkey {
        println!("[+] client identity verified");
    } else {
        println!("client: {:?}", client_pubkey);
        println!("remote: {:?}", remote_pubkey);

        panic!("unauthorized client public key");
    }

    let msg = responder.encrypt(b"hello from responder").unwrap();
    println!("encrypted: {:?}", msg);
    let msg = initiator.decrypt(&msg).unwrap();
    println!("decrypted: {:?}", String::from_utf8(msg).unwrap());

    let msg = initiator.encrypt(b"hello from initiator").unwrap();
    println!("encrypted: {:?}", msg);
    let msg = responder.decrypt(&msg).unwrap();
    println!("decrypted: {:?}", String::from_utf8(msg).unwrap());
}
