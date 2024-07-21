use async_std::io::{self, ReadExt, WriteExt};
use async_std::net::TcpStream;
use async_std::task;
use log::{error, info};
use clap::Parser;
use std::time::Duration;
use async_std::future::timeout;

mod config;
use config::Config;

async fn forward_data(
    mut read_stream: TcpStream,
    mut write_stream: TcpStream,
    src_addr: &str,
    dst_addr: &str,
) -> io::Result<()> {
    let mut buffer = [0u8; 4096];
    loop {
        // Устанавливаем таймаут на 5 секунд
        let result = timeout(Duration::from_secs(5), read_stream.read(&mut buffer)).await;
        // let result = read_stream.read(&mut buffer).await;

        match result {
            Ok(Ok(0)) => {
                info!("EOF reached from {} to {}", src_addr, dst_addr);
                break; // EOF
            }
            Ok(Ok(bytes_read)) => {
                if bytes_read > 0 {
                    info!("Forwarding {} bytes from {} to {}", bytes_read, src_addr, dst_addr);
                    if let Err(e) = write_stream.write_all(&buffer[..bytes_read]).await {
                        error!("Error writing to {}: {}", dst_addr, e);
                        return Err(e);
                    }
                }
            }
            Ok(Err(e)) => {
                error!("Error reading from {}: {}", src_addr, e);
                return Err(e);
            }
            Err(_) => {
                error!("Timeout while waiting for data from {}", src_addr);
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Timeout"));
            }
        }
    }
    Ok(())
}

async fn handle_connection(remote_addr: String, local_addr: String) -> io::Result<()> {
    info!("Create new connection...");

    // Подключаемся к локальному серверу
    let local_stream = match TcpStream::connect(&local_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to local server: {}", e);
            return Err(e);
        }
    };

    // Подключаемся к удаленному серверу
    let remote_stream = match TcpStream::connect(&remote_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to remote server: {}", e);
            return Err(e);
        }
    };

    // Запускаем задачи для обработки данных
    let local_stream_clone = local_stream.clone();
    let remote_stream_clone = remote_stream.clone();

    let remote_to_local = task::spawn(async move {
        if let Err(e) = forward_data(remote_stream_clone, local_stream_clone, "remote", "local").await {
            error!("Error forwarding data from remote to local: {}", e);
        }
    });

    let local_to_remote = task::spawn(async move {
        if let Err(e) = forward_data(local_stream, remote_stream, "local", "remote").await {
            error!("Error forwarding data from local to remote: {}", e);
        }
    });

    // Ждем завершения задач
    remote_to_local.await;
    local_to_remote.await;

    Ok(())
}

#[async_std::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    // Парсинг аргументов командной строки
    let config = Config::parse();
    let remote_addr = config.remote_addr;
    let local_addr = config.local_addr;

    info!("Starting client...");

    loop {
        // Подключаемся к удаленному серверу
        let remote_stream = match TcpStream::connect(&remote_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to remote server: {}", e);
                task::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // Подключаемся к локальному серверу
        let local_stream = match TcpStream::connect(&local_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to local server: {}", e);
                task::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("Connected to both remote and local servers");

        // Запускаем задачи для обработки данных
        let remote_addr_clone = remote_addr.clone();
        let local_addr_clone = local_addr.clone();

        task::spawn(async move {
            if let Err(e) = handle_connection(remote_addr_clone, local_addr_clone).await {
                error!("Error handling connection: {}", e);
            }
        });

        // Пауза перед следующей попыткой подключения, если предыдущие завершились
        task::sleep(Duration::from_secs(1)).await;
    }
}
