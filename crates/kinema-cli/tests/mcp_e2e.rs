use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

fn get_workspace_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../"))
}

struct McpTestClient {
    child: std::process::Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl McpTestClient {
    fn new() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kinema"))
            .current_dir(get_workspace_root())
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn kinema mcp");

        let stdout = child.stdout.take().expect("failed to capture stdout");
        let reader = BufReader::new(stdout);

        Self { child, reader }
    }

    fn send_request(&mut self, req: &serde_json::Value) {
        let stdin = self.child.stdin.as_mut().expect("failed to get stdin");
        let line = serde_json::to_string(req).unwrap();
        stdin.write_all(line.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
    }

    fn read_response(&mut self) -> serde_json::Value {
        let mut line = String::new();
        self.reader.read_line(&mut line).expect("failed to read line from MCP server");
        serde_json::from_str(&line).unwrap_or_else(|e| {
            panic!("failed to parse JSON from MCP response: {:?}, raw line: {:?}", e, line)
        })
    }
}

impl Drop for McpTestClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn test_mcp_e2e_lifecycle_and_tools() {
    let mut client = McpTestClient::new();

    // 1. initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }
    });
    client.send_request(&init_req);
    let init_resp = client.read_response();
    assert_eq!(init_resp["id"], 1);
    assert_eq!(init_resp["result"]["serverInfo"]["name"], "kinema");

    // Send initialized notification
    let initialized_notif = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    client.send_request(&initialized_notif);

    // 2. tools/list: must include 'check', must NOT include 'guide', 'explain', 'mcp'
    let list_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    client.send_request(&list_req);
    let list_resp = client.read_response();
    assert_eq!(list_resp["id"], 2);

    let tools = list_resp["result"]["tools"].as_array().expect("tools list must be array");
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().expect("tool name"))
        .collect();

    assert!(tool_names.contains(&"check"), "tools must contain 'check': {:?}", tool_names);
    assert!(tool_names.contains(&"fmt"), "tools must contain 'fmt'");
    assert!(tool_names.contains(&"ir"), "tools must contain 'ir'");
    assert!(tool_names.contains(&"netlist"), "tools must contain 'netlist'");
    assert!(tool_names.contains(&"rules"), "tools must contain 'rules'");
    assert!(tool_names.contains(&"graph"), "tools must contain 'graph'");
    assert!(tool_names.contains(&"gen-leaf"), "tools must contain 'gen-leaf'");
    assert!(tool_names.contains(&"doc_read"), "tools must contain 'doc_read'");
    assert!(tool_names.contains(&"doc_list"), "tools must contain 'doc_list'");

    assert!(!tool_names.contains(&"guide"), "tools must NOT contain 'guide'");
    assert!(!tool_names.contains(&"explain"), "tools must NOT contain 'explain'");
    assert!(!tool_names.contains(&"mcp"), "tools must NOT contain 'mcp'");

    // 3. tools/call check on examples/timer_core.v: isError: false, structuredContent.ok == true
    let call_check_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "check",
            "arguments": {
                "stage": "lint",
                "files": ["examples/timer_core.v"]
            }
        }
    });
    client.send_request(&call_check_req);
    let call_check_resp = client.read_response();
    assert_eq!(call_check_resp["id"], 3);

    let result = &call_check_resp["result"];
    let is_error = result["isError"].as_bool().unwrap_or(false);
    assert!(!is_error, "check tool call must not be error: {:?}", result);
    assert_eq!(result["structuredContent"]["ok"], true, "structuredContent.ok must be true");

    // 4. tools/call check with invalid/nonexistent argument: isError: true and error message from clap
    let invalid_call_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "check",
            "arguments": {
                "nonexistent_option": "value"
            }
        }
    });
    client.send_request(&invalid_call_req);
    let invalid_resp = client.read_response();
    assert_eq!(invalid_resp["id"], 4);

    let invalid_result = &invalid_resp["result"];
    assert_eq!(invalid_result["isError"], true, "Invalid argument must return isError: true");
    let error_text = invalid_result["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        error_text.contains("unexpected argument") || error_text.contains("nonexistent_option"),
        "Error message should mention clap validation reason: {}",
        error_text
    );

    // 5. tools/call doc_read with 'identity': internal links rewritten for MCP
    let doc_read_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "doc_read",
            "arguments": {
                "id": "identity"
            }
        }
    });
    client.send_request(&doc_read_req);
    let doc_read_resp = client.read_response();
    assert_eq!(doc_read_resp["id"], 5);

    let doc_content = doc_read_resp["result"]["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        doc_content.contains("kinema://doc/rules/identity-missing")
            || doc_content.contains("kinema://doc/identity-missing")
            || doc_content.contains("see: 'kinema explain identity-missing'")
            || doc_content.contains("identity-missing"),
        "doc_read content must contain link to identity-missing: {}",
        doc_content
    );

    // 6. tools/call check with file containing syntax error: diagnostic hint must format for MCP
    let bad_file = get_workspace_root().join("target/bad_syntax_test.v");
    std::fs::write(&bad_file, "module bad (;\n").unwrap();
    let check_bad_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "check",
            "arguments": {
                "files": [bad_file.to_str().unwrap()]
            }
        }
    });
    client.send_request(&check_bad_req);
    let check_bad_resp = client.read_response();
    let _ = std::fs::remove_file(&bad_file);
    assert_eq!(check_bad_resp["id"], 6);
    let check_bad_result = &check_bad_resp["result"];
    assert_eq!(check_bad_result["isError"], true);
    let bad_output_text = check_bad_result["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        bad_output_text.contains("Help: call tool 'doc_read {\"id\": \"syntax-error\"}'"),
        "Diagnostic hint in MCP must use Mcp render mode, but got: {}",
        bad_output_text
    );
    assert!(
        !bad_output_text.contains("kinema explain"),
        "Diagnostic hint in MCP must NOT suggest 'kinema explain', but got: {}",
        bad_output_text
    );
}
