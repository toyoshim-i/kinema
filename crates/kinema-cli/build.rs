use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    std::env::set_current_dir(workspace_root).unwrap();
    agent_doc::build::embed("docs").expect("failed to embed documentation pages");
}
