use clap::Parser;

/// Program arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Remote address to connect to
    #[arg(short, long, default_value = "127.0.0.1:9000")]
    pub remote_addr: String,

    /// Local address to connect to
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    pub local_addr: String,
}
