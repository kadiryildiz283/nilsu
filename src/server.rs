use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct ContextRequest {
    action: String,
    file: String,
    cursor_line: usize,
}

use crate::parser::ParsedContext;

#[derive(Debug, Serialize)]
struct ContextResponse {
    status: String,
    request_id: String,
    message: String,
    context_snippet: Option<ParsedContext>,
    latency_ms: u128,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    error: String,
}

pub async fn start_daemon(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = config.socket_path.clone();
    
    // Eski soket dosyasını temizle
    if Path::new(&socket_path).exists() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!(r#"{{"event":"server_started","socket_path":"{}"}}"#, socket_path);

    // Geri basınç (Backpressure) kapısı: Kontrolsüz task üretimini engeller
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_connections));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                // Kapasite kontrolü (Fast-Fail): Yer yoksa kuyruğa alma, direkt reject et
                let permit = match semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => {
                        println!(r#"{{"event":"request_rejected","reason":"max_connections_reached"}}"#);
                        continue; 
                    }
                };

                let timeout_ms = config.timeout_ms;
                tokio::spawn(async move {
                    handle_client(stream, permit, timeout_ms).await;
                });
            }
            Err(e) => {
                println!(r#"{{"event":"accept_error","message":"{}"}}"#, e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, _permit: OwnedSemaphorePermit, timeout_ms: u64) {
    let start_time = Instant::now();
    let mut buffer = vec![0; 1024];

    // İstek işleme adımını korumalı timeout içine alıyoruz
    let result = timeout(Duration::from_millis(timeout_ms), async {
        match stream.read(&mut buffer).await {
            Ok(0) => return, // Bağlantı kapandı
            Ok(n) => {
                let raw_request = String::from_utf8_lossy(&buffer[..n]);
                
                // Stateless parsing ve çökme koruması (Crash-proof)
                let req: ContextRequest = match serde_json::from_str(&raw_request) {
                    Ok(r) => r,
                    Err(_) => {
                        let err_resp = ErrorResponse { status: "error".to_string(), error: "malformed_json".to_string() };
                        let _ = stream.write_all(serde_json::to_string(&err_resp).unwrap().as_bytes()).await;
                        println!(r#"{{"event":"malformed_request_received"}}"#);
                        return;
                    }
                };

                // Yapılandırılmış JSON Log çıktısı (stdout)
                println!(
                    r#"{{"event":"request_received","action":"{}","file":"{}","cursor":{}}}"#,
                    req.action, req.file, req.cursor_line
                );

                // Parse context using tree-sitter
                let context_snippet = crate::parser::get_context(&req.file, req.cursor_line);

                // V1 Dummy Başarı Yanıtı (UUID üretimi dahil)
                let latency = start_time.elapsed().as_millis();
                let response = ContextResponse {
                    status: "ok".to_string(),
                    request_id: Uuid::new_v4().to_string(),
                    message: if context_snippet.is_some() { "context extracted".to_string() } else { "no context found".to_string() },
                    context_snippet,
                    latency_ms: latency,
                };

                let response_bytes = serde_json::to_string(&response).unwrap();
                let _ = stream.write_all(response_bytes.as_bytes()).await;
            }
            Err(e) => {
                println!(r#"{{"event":"read_error","message":"{}"}}"#, e);
            }
        }
    }).await;

    // Timeout ihlali durumunda sunucunun kilitlenmesini engelle
    if result.is_err() {
        let timeout_resp = ErrorResponse { status: "error".to_string(), error: "timeout".to_string() };
        let _ = stream.write_all(serde_json::to_string(&timeout_resp).unwrap().as_bytes()).await;
        println!(r#"{{"event":"request_timeout_triggered"}}"#);
    }
}
