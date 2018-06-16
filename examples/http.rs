extern crate snail;
extern crate env_logger;

fn main() {
    env_logger::init();

    let resolver = snail::dns::Resolver::cloudflare();
    let client = snail::web::Client::new(resolver);
    let reply = client.get("https://httpbin.org/anything");
    println!("{:?}", reply);
}
