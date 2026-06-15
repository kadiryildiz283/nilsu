use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let payload = r#"{"action": "get_context", "file": "src/main.rs", "cursor_line": 5}"#;
    println!("Sending request: {}", payload);

    let mut stream = UnixStream::connect("/tmp/nilsu.sock").await?;
    stream.write_all(payload.as_bytes()).await?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;

    let resp_str = String::from_utf8(response)?;
    println!("\nResponse from server:");
    println!("{}", resp_str);

    // Pretty-print json if possible
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&resp_str) {
        println!("\nPretty printed response:");
        println!("{}", serde_json::to_string_pretty(&val)?);
    }

    Ok(())
}
