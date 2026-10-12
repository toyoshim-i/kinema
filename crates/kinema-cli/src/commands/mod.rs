pub mod check;
pub mod common;
pub mod fmt;
pub mod gen_leaf;
pub mod graph;
pub mod ir;
pub mod netlist;
pub mod rules;

use serde_json::Value;
use std::process::ExitCode;

#[derive(Debug, Clone, Default)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub data: Option<Value>,
}

impl CommandResult {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: String::new(),
            success: true,
            data: None,
        }
    }

    pub fn error(stderr: impl Into<String>) -> Self {
        Self {
            stdout: String::new(),
            stderr: stderr.into(),
            success: false,
            data: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn print(&self) {
        if !self.stdout.is_empty() {
            print!("{}", self.stdout);
        }
        if !self.stderr.is_empty() {
            eprint!("{}", self.stderr);
        }
    }

    pub fn exit_code(&self) -> ExitCode {
        if self.success {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        }
    }

    pub fn to_tool_result(&self) -> agent_doc::mcp::ToolResult {
        let text = if !self.stdout.is_empty() {
            &self.stdout
        } else {
            &self.stderr
        };
        if self.success {
            if let Some(ref data) = self.data {
                agent_doc::mcp::ToolResult::structured(text, data.clone())
            } else {
                agent_doc::mcp::ToolResult::text(text)
            }
        } else {
            let mut res = agent_doc::mcp::ToolResult::error(text);
            if let Some(ref data) = self.data {
                res.structured_content = Some(data.clone());
            }
            res
        }
    }
}
