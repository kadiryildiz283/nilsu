mod config;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Konfigürasyonu yükle
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!(r#"{{"status":"error","error":"config_load_failed","message":"{}"}}"#, e);
            std::process::exit(1);
        }
    };

    // 2. Çekirdek Daemon'ı başlat
    if let Err(e) = server::start_daemon(config).await {
        eprintln!(r#"{{"status":"error","error":"daemon_runtime_failed","message":"{}"}}"#, e);
        std::process::exit(1);
    }

    Ok(())
}
