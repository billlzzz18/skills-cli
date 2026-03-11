use std::env;
use std::io::{self, Read, Write};
use std::process::{Command, exit};

use serde::{Deserialize, Serialize};
use serde_json::{Value};

const ALLOWLISTED_TERMINALS: [&str; 6] = [
    "ghostty",
    "iterm.app",
    "iterm2",
    "kitty",
    "vscode",
    "apple_terminal",
];

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
    Response(JsonRpcResponse),
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    result: Option<Value>,
    error: Option<Value>,
}

fn main() {

    let mut payload = String::new();
    io::stdin().read_to_string(&mut payload).unwrap();

    if payload.trim().is_empty() {
        exit(0);
    }

    let parsed: Value = match serde_json::from_str(&payload) {
        Ok(v) => v,
        Err(_) => {
            exit(0);
        }
    };

    // ------------------------------------------------
    // JSON-RPC support
    // ------------------------------------------------

    if parsed.get("jsonrpc").is_some() {

        if let Ok(msg) = serde_json::from_value::<JsonRpcMessage>(parsed.clone()) {

            match msg {

                JsonRpcMessage::Notification(n) => {

                    if n.method == "notify" {
                        let message = extract_message(&n.params);
                        notify("Gemini CLI", &message);
                    }

                }

                JsonRpcMessage::Request(r) => {

                    if r.method == "notify" {

                        let message = extract_message(&r.params);
                        notify("Gemini CLI", &message);

                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: r.id,
                            result: Some(Value::String("ok".to_string())),
                            error: None,
                        };

                        let out = serde_json::to_string(&response).unwrap();
                        println!("{}", out);
                    }

                }

                JsonRpcMessage::Response(_) => {}
            }
        }

        return;
    }

    // ------------------------------------------------
    // Original behaviour (unchanged)
    // ------------------------------------------------

    if parsed["notification_type"] != "ToolPermission" {
        exit(0);
    }

    let message = parsed["message"]
        .as_str()
        .unwrap_or("Gemini Agent requires attention");

    notify("Gemini CLI", message);
}

fn extract_message(params: &Option<Value>) -> String {

    if let Some(p) = params {
        if let Some(obj) = p.as_object() {
            if let Some(msg) = obj.get("message") {
                if let Some(s) = msg.as_str() {
                    return s.to_string();
                }
            }
        }
    }

    "Gemini Agent requires attention".to_string()
}

fn notify(title: &str, message: &str) {

    let terminal_success = try_terminal_notification(title, message);

    if !terminal_success {
        try_os_notification(title, message);
    }
}

fn try_terminal_notification(_title: &str, message: &str) -> bool {

    let term_program = env::var("TERM_PROGRAM")
        .unwrap_or_else(|_| "".to_string())
        .to_lowercase();

    let mut is_supported = false;

    for term in ALLOWLISTED_TERMINALS.iter() {
        if term_program.contains(term) {
            is_supported = true;
        }
    }

    if is_supported {

        let mut stdout = io::stdout();

        let seq = format!("\x1b]9;{}\x07", message);

        stdout.write_all(seq.as_bytes()).unwrap();
        stdout.flush().unwrap();

        return true;
    }

    false
}

fn try_os_notification(title: &str, message: &str) -> bool {

    let platform = env::consts::OS;

    if platform == "macos" {
        return notify_macos(title, message);
    }

    if platform == "linux" {
        return notify_linux(title, message);
    }

    false
}

fn notify_macos(title: &str, message: &str) -> bool {

    let check = Command::new("which")
        .arg("osascript")
        .output();

    if check.is_err() {
        return false;
    }

    let safe_title = title.replace("\"", "\\\"");
    let safe_message = message.replace("\"", "\\\"");

    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        safe_message,
        safe_title
    );

    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

fn notify_linux(title: &str, message: &str) -> bool {

    let check = Command::new("which")
        .arg("notify-send")
        .output();

    if check.is_err() {
        return false;
    }

    let safe_title = title.replace("\"", "\\\"");
    let safe_message = message.replace("\"", "\\\"");

    let status = Command::new("notify-send")
        .arg(safe_title)
        .arg(safe_message)
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}