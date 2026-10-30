use clap::{Parser, CommandFactory, error::ErrorKind};
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(version = "1.0", about = "Асинхронный сканнер портов", long_about = None)]
pub struct Args {
    pub target: IpAddr,

    #[arg(short = 's', long, default_value_t = 1)]
    pub start_port: u16,

    #[arg(short = 'e', long, default_value_t = 1000)]
    pub end_port: u16,

    #[arg(short = 'c', long, default_value_t = 500)]
    pub concurrency: usize,

    #[arg(short = 't', long, default_value_t = 100)]
    pub timeout: u64,
}

impl Args {
    pub fn validate(self) -> Result<Self, clap::Error> {
        if self.start_port > self.end_port {
            let mut cmd = Self::command();
            return Err(cmd.error(
                ErrorKind::ArgumentConflict,
                format!(
                    "Ошибка: Начальный порт (-s: {}) не может быть больше конечного (-e: {})!",
                    self.start_port, self.end_port
                )
            ));
        }
        Ok(self)
    }
}

pub fn parse_args() -> Args {
    match Args::parse().validate() {
        Ok(valid_args) => valid_args,
        Err(clap_error) => clap_error.exit(),
    }
}
