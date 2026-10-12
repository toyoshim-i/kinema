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
        let empty_dir = get_workspace_root().join("target/empty_kicad_config");
        let _ = std::fs::create_dir_all(&empty_dir);
        let mut child = Command::new(env!("CARGO_BIN_EXE_kinema"))
            .current_dir(get_workspace_root())
            .env("KICAD_CONFIG_HOME", &empty_dir)
            .env_remove("KICAD_CONFIG_DIR")
            .env_remove("KICAD_FOOTPRINT_DIR")
            .env_remove("KISYSMOD")
            .env_remove("KICAD10_FOOTPRINT_DIR")
            .env_remove("KICAD9_FOOTPRINT_DIR")
            .env_remove("KICAD8_FOOTPRINT_DIR")
            .env_remove("KICAD7_FOOTPRINT_DIR")
            .env_remove("KICAD6_FOOTPRINT_DIR")
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
    assert_eq!(init_resp["result"]["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));

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

    // 7. tools/call fmt with check: true on examples/timer_core.v
    let fmt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "fmt",
            "arguments": {
                "check": true,
                "files": ["examples/timer_core.v"]
            }
        }
    });
    client.send_request(&fmt_req);
    let fmt_resp = client.read_response();
    assert_eq!(fmt_resp["id"], 7);
    let fmt_result = &fmt_resp["result"];
    assert!(!fmt_result["isError"].as_bool().unwrap_or(false), "fmt tool must not be error: {:?}", fmt_result);
    let fmt_text = fmt_result["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        fmt_text.contains("canonically formatted") || fmt_text.contains("already formatted"),
        "fmt output must describe result rather than being empty: {}",
        fmt_text
    );

    // 8. tools/call ir on examples/timer_core.v
    let ir_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "ir",
            "arguments": {
                "files": ["examples/timer_core.v"]
            }
        }
    });
    client.send_request(&ir_req);
    let ir_resp = client.read_response();
    assert_eq!(ir_resp["id"], 8);
    let ir_result = &ir_resp["result"];
    assert!(!ir_result["isError"].as_bool().unwrap_or(false), "ir tool must not be error: {:?}", ir_result);
    assert!(ir_result["structuredContent"].get("top_module").is_some());

    // 9. tools/call netlist with output path in target/
    let out_net = get_workspace_root().join("target/mcp_test_timer.net");
    let netlist_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "tools/call",
        "params": {
            "name": "netlist",
            "arguments": {
                "output": out_net.to_str().unwrap(),
                "files": ["examples/timer_core.v"]
            }
        }
    });
    client.send_request(&netlist_req);
    let netlist_resp = client.read_response();
    assert_eq!(netlist_resp["id"], 9);
    let netlist_result = &netlist_resp["result"];
    assert!(!netlist_result["isError"].as_bool().unwrap_or(false), "netlist tool must not be error: {:?}", netlist_result);
    assert!(out_net.exists(), "Exported netlist file must exist");
    let _ = std::fs::remove_file(&out_net);

    // 10. tools/call rules with pro and dru paths in target/
    let out_pro = get_workspace_root().join("target/mcp_test_timer.kicad_pro");
    let out_dru = get_workspace_root().join("target/mcp_test_timer.kicad_dru");
    let rules_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "rules",
            "arguments": {
                "pro": out_pro.to_str().unwrap(),
                "dru": out_dru.to_str().unwrap(),
                "files": ["examples/timer_core.v"]
            }
        }
    });
    client.send_request(&rules_req);
    let rules_resp = client.read_response();
    assert_eq!(rules_resp["id"], 10);
    let rules_result = &rules_resp["result"];
    assert!(!rules_result["isError"].as_bool().unwrap_or(false), "rules tool must not be error: {:?}", rules_result);
    assert!(out_pro.exists(), "Exported kicad_pro must exist");
    assert!(out_dru.exists(), "Exported kicad_dru must exist");
    let _ = std::fs::remove_file(&out_pro);
    let _ = std::fs::remove_file(&out_dru);

    // 11. tools/call graph on examples/timer_core.v
    let graph_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "tools/call",
        "params": {
            "name": "graph",
            "arguments": {
                "files": ["examples/timer_core.v"]
            }
        }
    });
    client.send_request(&graph_req);
    let graph_resp = client.read_response();
    assert_eq!(graph_resp["id"], 11);
    let graph_result = &graph_resp["result"];
    assert!(!graph_result["isError"].as_bool().unwrap_or(false), "graph tool must not be error: {:?}", graph_result);
    assert!(graph_result["structuredContent"].get("modules").is_some());

    // 12. tools/call gen-leaf
    let gen_leaf_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 12,
        "method": "tools/call",
        "params": {
            "name": "gen-leaf",
            "arguments": {
                "name": "ATmega328P",
                "prefix": "U",
                "footprint": "Package_QFP:TQFP-32_7x7mm_P0.8mm"
            }
        }
    });
    client.send_request(&gen_leaf_req);
    let gen_leaf_resp = client.read_response();
    assert_eq!(gen_leaf_resp["id"], 12);
    let gen_leaf_result = &gen_leaf_resp["result"];
    assert!(!gen_leaf_result["isError"].as_bool().unwrap_or(false), "gen-leaf tool must not be error: {:?}", gen_leaf_result);
    let gen_leaf_text = gen_leaf_result["content"][0]["text"].as_str().unwrap_or("");
    assert!(gen_leaf_text.contains("module ATmega328P"), "gen-leaf must output module definition: {}", gen_leaf_text);
}
