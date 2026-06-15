use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::Semaphore;

async fn send_request(payload: &str) -> Result<(Duration, String), String> {
    let start = Instant::now();
    let mut stream = match UnixStream::connect("/tmp/nilsu.sock").await {
        Ok(s) => s,
        Err(e) => return Err(format!("Connection error: {}", e)),
    };

    if let Err(e) = stream.write_all(payload.as_bytes()).await {
        return Err(format!("Write error: {}", e));
    }

    let mut response = Vec::new();
    if let Err(e) = stream.read_to_end(&mut response).await {
        return Err(format!("Read error: {}", e));
    }

    let duration = start.elapsed();
    let resp_str = String::from_utf8(response).unwrap_or_default();
    Ok((duration, resp_str))
}

#[tokio::main]
async fn main() {
    println!("Starting Benchmark Stress Test...");
    println!("Target: 1000 requests, Concurrency limit: 100");

    let total_requests = 1000;
    let concurrency_limit = 100;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let mut handles = Vec::with_capacity(total_requests);

    let start_test = Instant::now();

    for i in 0..total_requests {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        // 90% valid JSON, 10% malformed JSON
        let is_malformed = i % 10 == 0;
        let payload = if is_malformed {
            "{invalid_json: true,".to_string()
        } else {
            format!(
                r#"{{"action": "get_context", "file": "test.rs", "cursor_line": {}}}"#,
                i
            )
        };

        handles.push(tokio::spawn(async move {
            let res = send_request(&payload).await;
            drop(permit);
            res
        }));
    }

    let mut latencies = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok((duration, response_str))) => {
                latencies.push(duration.as_micros() as u64);
                if response_str.contains(r#""status":"ok""#) {
                    success_count += 1;
                } else {
                    error_count += 1;
                }
            }
            Ok(Err(_err)) => {
                error_count += 1;
            }
            Err(_) => {
                error_count += 1;
            }
        }
    }

    let total_duration = start_test.elapsed();

    println!("\n================ STRESS TEST RESULTS ================");
    println!("Total Duration: {:?}", total_duration);
    println!("Successful Requests (status: ok): {}", success_count);
    println!("Failed/Malformed/Rejected Requests: {}", error_count);

    if !latencies.is_empty() {
        latencies.sort();
        let len = latencies.len();
        
        let sum: u64 = latencies.iter().sum();
        let mean = (sum as f64 / len as f64) / 1000.0;

        let p50 = latencies[(len * 50) / 100] as f64 / 1000.0;
        let p90 = latencies[(len * 90) / 100] as f64 / 1000.0;
        let p95 = latencies[(len * 95) / 100] as f64 / 1000.0;
        let p99 = latencies[(len * 99) / 100] as f64 / 1000.0;

        println!("Mean Latency: {:.4} ms", mean);
        println!("P50 (Median) Latency: {:.4} ms", p50);
        println!("P90 Latency: {:.4} ms", p90);
        println!("P95 Latency: {:.4} ms", p95);
        println!("P99 Latency: {:.4} ms", p99);
    } else {
        println!("No latencies recorded. All requests might have failed to connect.");
    }
    println!("=====================================================");
}
