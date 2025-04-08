use std::path::Path;

use manager::Manager;

mod bindings;
mod config;
mod manager;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use trigger::AbortController;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let manager = Manager::open(Path::new("./triggerman/tasks").canonicalize().unwrap());

    let abort = AbortController::default();

    let abort_clone = abort.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await;
        abort_clone.trigger();
    });

    manager.run(abort).await;
}
