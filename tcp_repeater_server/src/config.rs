use clap::Parser;

/// Структура для конфигурации приложения
#[derive(Parser)]
#[command(name = "tcp_forwarder")]
#[command(about = "A simple TCP forwarder", long_about = None)]
pub struct Config {
    /// Адрес для прослушивания внутренних соединений
    #[arg(short, long, default_value = "0.0.0.0:9000")]
    pub client_listener_addr: String,

    /// Адрес для прослушивания внешних соединений
    #[arg(short, long, default_value = "0.0.0.0:8000")]
    pub external_listener_addr: String,
}
