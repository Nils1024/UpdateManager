use update_manager::comm::server::{Server};
use update_manager::util::config::get_config;

fn main() {
    let server = Server::new(format!("127.0.0.1:{}", get_config().get("port").unwrap()));

    println!("Server started at {}", get_config().get("port").unwrap());
    println!("Hash: {}", server.get_hash());

    server.start().expect("Server stopped");
}