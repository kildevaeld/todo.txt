use std::path::Path;

use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName, traits::tokio::Listener,
};
use manager::Manager;

mod bindings;
mod config;
mod manager;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use trigger::AbortController;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    try_join,
};

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

async fn start_server(abort: AbortController) -> color_eyre::Result<()> {
    let printname = "triggerman.sock";
    let socket_name = printname.to_ns_name::<GenericNamespaced>()?;

    let listener = ListenerOptions::new().name(socket_name).create_tokio()?;

    loop {
        let conn = match listener.accept().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("There was an error with an incoming connection: {e}");
                continue;
            }
        };

        let mut reader = BufReader::new(&conn);

        let mut buffer = String::with_capacity(128);
        reader.read_line(&mut buffer).await?;
    }

    Ok(())
}
