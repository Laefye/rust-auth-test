use network::{database::PoloDB, Network};

mod network;

fn main() {
    let network = Network::new(PoloDB::new());
    let user_manager = network.user_manager();
    let result = user_manager.login("laegfye".to_string(), "123".to_string()).unwrap();
}