//! Stdio MCP (Model Context Protocol) server with telescope-style search tools,
//! started by the Zed extension for the agent panel.

mod fuzzy;
mod tools;

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const SERVER_NAME: &str = "telescope-zed-mcp";
const DEFAULT_PROTOCOL: &str = "2025-06-18";

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(msg) => handle(&msg),
            Err(e) => Some(error(Value::Null, -32700, &format!("parse error: {e}"))),
        };
        if let Some(reply) = reply {
            if writeln!(stdout, "{reply}").and_then(|_| stdout.flush()).is_err() {
                break;
            }
        }
    }
}

/// Reply for one JSON-RPC message; notifications get none.
fn handle(msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned()?;
    let method = msg.get("method").and_then(Value::as_str).unwrap_or_default();
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => {
            let version = params.get("protocolVersion").and_then(Value::as_str).unwrap_or(DEFAULT_PROTOCOL);
            ok(id, json!({
                "protocolVersion": version,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
                "instructions": "Telescope-style project search. Pass the project root as `path`."
            }))
        }
        "ping" => ok(id, json!({})),
        "tools/list" => ok(id, json!({ "tools": tools::definitions() })),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            match tools::call(name, &args) {
                Ok(text) => ok(id, json!({ "content": [{ "type": "text", "text": text }], "isError": false })),
                Err(e) => ok(id, json!({ "content": [{ "type": "text", "text": e }], "isError": true })),
            }
        }
        _ => error(id, -32601, &format!("method not found: {method}")),
    })
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_echoes_the_client_protocol() {
        let reply = handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2024-11-05" } })).unwrap();
        assert_eq!(reply["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(reply["result"]["serverInfo"]["name"], SERVER_NAME);
    }

    #[test]
    fn notifications_get_no_reply() {
        assert!(handle(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).is_none());
    }

    #[test]
    fn unknown_method_is_a_json_rpc_error() {
        let reply = handle(&json!({ "jsonrpc": "2.0", "id": 7, "method": "nope" })).unwrap();
        assert_eq!(reply["error"]["code"], -32601);
        assert_eq!(reply["id"], 7);
    }

    #[test]
    fn tool_failure_is_reported_in_the_result() {
        let reply = handle(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": { "name": "nope", "arguments": { "path": "/nonexistent-telescope-root" } } })).unwrap();
        assert_eq!(reply["result"]["isError"], true);
    }
}
