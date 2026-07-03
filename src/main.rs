use rosetta::{e384_commands::connect_to_first_device, syncro::SyncroV1};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    tracing::info!("app started");

    let d = connect_to_first_device();
    match d {
        Ok(r) => println!("CONNECTED"),
        _ => println!("NOOOOO"),
    }
    let s = SyncroV1::from_file("src\\assets\\syncropatch.toml");
    if let Ok(syncro) = s {
        syncro.apply_complete_calibration();
    }
}
