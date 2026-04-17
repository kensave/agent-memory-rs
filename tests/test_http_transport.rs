use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::net::TcpStream;

const BIN: &str = env!("CARGO_BIN_EXE_agent-memory-mcp");

fn send_initialize(stdin: &mut impl Write) {
    let msg = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    writeln!(stdin, "{}", msg).unwrap();
    stdin.flush().unwrap();
}

fn read_response(stdout: &mut impl BufRead) -> serde_json::Value {
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn wait_for_port(port: u16, timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

#[test]
fn test_stdio_initialize() {
    let mut child = Command::new(BIN)
        .arg("test-stdio-init")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start server");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    send_initialize(&mut stdin);
    let resp = read_response(&mut stdout);

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(resp["result"]["protocolVersion"].is_string());
    assert!(resp["result"]["capabilities"]["tools"].is_object());

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn test_stdio_learn_and_search() {
    let mut child = Command::new(BIN)
        .arg("test-stdio-learn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start server");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // Initialize
    send_initialize(&mut stdin);
    let _ = read_response(&mut stdout);

    // Send initialized notification
    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).unwrap();
    stdin.flush().unwrap();

    // Learn
    let learn = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"learn","arguments":{"text":"Rust is a systems programming language","importance_score":0.9,"tags":"test,lang"}}}"#;
    writeln!(stdin, "{}", learn).unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    assert_eq!(resp["id"], 2);
    let content: serde_json::Value = serde_json::from_str(
        resp["result"]["content"][0]["text"].as_str().unwrap()
    ).unwrap();
    assert_eq!(content["status"], "success");

    // Search
    let search = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search","arguments":{"query":"programming language","limit":5}}}"#;
    writeln!(stdin, "{}", search).unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    assert_eq!(resp["id"], 3);
    let content: serde_json::Value = serde_json::from_str(
        resp["result"]["content"][0]["text"].as_str().unwrap()
    ).unwrap();
    assert!(content["count"].as_u64().unwrap() > 0);

    drop(stdin);
    child.wait().unwrap();
}

fn parse_sse_response(body: &str) -> serde_json::Value {
    // SSE format can have multiple "data: " lines, we want the last JSON one
    for line in body.lines().rev() {
        let trimmed = line.strip_prefix("data: ").unwrap_or(line).trim();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if json.get("jsonrpc").is_some() {
                return json;
            }
        }
    }
    panic!("No valid JSON-RPC response found in: {}", body);
}

#[test]
fn test_http_initialize() {
    let port = 18231u16;

    let mut child = Command::new(BIN)
        .args(["--http", &format!("127.0.0.1:{}", port), "test-http-init"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start HTTP server");

    assert!(wait_for_port(port, Duration::from_secs(10)), "HTTP server didn't start");

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{}/mcp", port))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#)
        .send()
        .expect("HTTP request failed");

    assert_eq!(resp.status(), 200);
    let body = resp.text().unwrap();
    // Streamable HTTP returns SSE format: "data: {...}\n\n"
    let json = parse_sse_response(&body);
    assert_eq!(json["jsonrpc"], "2.0");
    assert!(json["result"]["capabilities"]["tools"].is_object());

    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn test_http_learn_and_search() {
    let port = 18232u16;

    let mut child = Command::new(BIN)
        .args(["--http", &format!("127.0.0.1:{}", port), "test-http-learn"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start HTTP server");

    assert!(wait_for_port(port, Duration::from_secs(10)), "HTTP server didn't start");

    let client = reqwest::blocking::Client::new();
    let base = format!("http://127.0.0.1:{}/mcp", port);

    // Initialize session
    let resp = client
        .post(&base)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#)
        .send()
        .unwrap();

    let session_id = resp.headers()
        .get("mcp-session-id")
        .map(|v| v.to_str().unwrap().to_string());

    // Send initialized notification
    let mut req = client
        .post(&base)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
    if let Some(ref sid) = session_id {
        req = req.header("mcp-session-id", sid);
    }
    req.send().unwrap();

    // Learn
    let mut req = client
        .post(&base)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"learn","arguments":{"text":"Costa Rica is known for pura vida","importance_score":0.8,"tags":"test"}}}"#);
    if let Some(ref sid) = session_id {
        req = req.header("mcp-session-id", sid);
    }
    let resp = req.send().unwrap();
    let body = resp.text().unwrap();
    let json = parse_sse_response(&body);
    let content: serde_json::Value = serde_json::from_str(
        json["result"]["content"][0]["text"].as_str().unwrap()
    ).unwrap();
    assert_eq!(content["status"], "success");

    // Search
    let mut req = client
        .post(&base)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search","arguments":{"query":"pura vida","limit":5}}}"#);
    if let Some(ref sid) = session_id {
        req = req.header("mcp-session-id", sid);
    }
    let resp = req.send().unwrap();
    let body = resp.text().unwrap();
    let json = parse_sse_response(&body);
    let content: serde_json::Value = serde_json::from_str(
        json["result"]["content"][0]["text"].as_str().unwrap()
    ).unwrap();
    assert!(content["count"].as_u64().unwrap() > 0);

    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn test_stdio_backward_compatible_args() {
    // Original usage: agent-memory-mcp workspace-name (positional arg, no flags)
    let mut child = Command::new(BIN)
        .arg("test-compat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start server with positional arg");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    send_initialize(&mut stdin);
    let resp = read_response(&mut stdout);
    assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");

    drop(stdin);
    child.wait().unwrap();
}
