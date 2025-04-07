use std::path::Path;

use manager::Manager;

mod bindings;
mod config;
mod manager;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let manager = Manager::open(Path::new("./triggerman/tasks").canonicalize().unwrap());

    manager.run().await;
}
