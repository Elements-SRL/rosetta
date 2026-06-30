use rosetta::{get_ram, read_eeprom};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    tracing::info!("app started");
    read_eeprom();
    get_ram(0);
}
