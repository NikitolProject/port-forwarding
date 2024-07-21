use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use log::{error, info};
use std::collections::VecDeque;
use clap::Parser;

mod config;
use config::Config;

async fn forward_data(
    mut source: TcpStream,
    mut destination: TcpStream,
    src_name: &str,
    dst_name: &str,
) -> io::Result<()> {
    let mut buffer = vec![0u8; 4096];
    loop {
        match source.read(&mut buffer).await {
            Ok(0) => {
                info!("{} closed the connection", src_name);
                break;
            }
            Ok(n) => {
                info!("Forwarding {} bytes from {} to {}", n, src_name, dst_name);
                if let Err(e) = destination.write_all(&buffer[..n]).await {
                    error!("Error writing from {} to {}: {}", src_name, dst_name, e);
                    break;
                }
            }
            Err(e) => {
                error!("Error reading from {}: {}", src_name, e);
                break;
            }
        }
    }
    if let Err(e) = destination.shutdown(std::net::Shutdown::Both) {
        error!("Error shutting down {} stream: {}", dst_name, e);
    }
    Ok(())
}

async fn handle_external_client(
    external_client: TcpStream,
    internal_clients: Arc<Mutex<VecDeque<TcpStream>>>,
) -> io::Result<()> {
    info!("External client waiting for an internal client...");

    let internal_client = {
        let mut internal_clients = internal_clients.lock().await;
        loop {
            if let Some(mut client) = internal_clients.pop_back() {
                match client.write(&[0]).await {
                    Ok(_) => {
                        // Проверяем, что соединение активно
                        break Some(client);
                    }
                    Err(e) => {
                        // Соединение закрыто или недоступно
                        info!("Skipping closed internal client connection: {}", e);
                    }
                }
            } else {
                break None;
            }
        }
    };

    if let Some(internal_client_stream) = internal_client {
        info!("Internal client assigned to external client");

        let external_client_clone = external_client.clone();
        let internal_client_clone = internal_client_stream.clone();

        let forward_external_to_internal = forward_data(
            external_client_clone,
            internal_client_clone,
            "external client",
            "internal client",
        );

        let forward_internal_to_external = forward_data(
            internal_client_stream,
            external_client,
            "internal client",
            "external client",
        );

        let result_external_to_internal = task::spawn(async move {
            forward_external_to_internal.await
        });

        let result_internal_to_external = task::spawn(async move {
            forward_internal_to_external.await
        });

        if let Err(e) = result_external_to_internal.await {
            error!("Error forwarding data from external to internal: {}", e);
        }
        if let Err(e) = result_internal_to_external.await {
            error!("Error forwarding data from internal to external: {}", e);
        }
    } else {
        error!("No internal clients available");
    }

    Ok(())
}

async fn accept_internal_connections(
    internal_listener: TcpListener,
    internal_clients: Arc<Mutex<VecDeque<TcpStream>>>,
) -> io::Result<()> {
    loop {
        match internal_listener.accept().await {
            Ok((stream, _)) => {
                info!("Internal client connected");
                let mut internal_clients_lock = internal_clients.lock().await;
                internal_clients_lock.clear();
                internal_clients_lock.push_back(stream);
                info!("Updating the current free internal connection");
            }
            Err(e) => error!("Failed to accept internal client connection: {}", e),
        }
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let config = Config::parse();
    let internal_address = &config.client_listener_addr;
    let external_address = &config.external_listener_addr;

    let external_listener = TcpListener::bind(external_address).await?;
    let internal_listener = TcpListener::bind(internal_address).await?;

    let internal_clients = Arc::new(Mutex::new(VecDeque::new()));

    let internal_clients_clone = Arc::clone(&internal_clients);

    info!("Listening for internal clients on {}", internal_address);

    task::spawn(async move {
        if let Err(e) = accept_internal_connections(internal_listener, internal_clients_clone).await {
            error!("Error in internal connection handler: {}", e);
        }
    });

    info!("Listening for external clients on {}", external_address);

    loop {
        let (external_client, _) = external_listener.accept().await?;
        info!("Accepted connection from external client");

        let internal_clients_clone = Arc::clone(&internal_clients);
        task::spawn(async move {
            if let Err(e) = handle_external_client(external_client, internal_clients_clone).await {
                error!("Error handling external client: {}", e);
            }
        });
    }
}
