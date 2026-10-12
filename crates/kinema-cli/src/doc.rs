use agent_doc::DocEngine;

pub fn build_doc_engine() -> DocEngine {
    let mut engine = DocEngine::new("kinema")
        .with_subcommands("guide", "explain");

    let pages: &[(&str, &str)] = &include!(concat!(env!("OUT_DIR"), "/agent_doc_pages.rs"));
    engine.add_pages(pages.iter().copied());

    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinema_lint::codes;

    #[test]
    fn test_all_diagnostic_codes_have_documentation_pages() {
        let engine = build_doc_engine();
        let missing = engine.missing_pages(codes::ALL);
        assert!(
            missing.is_empty(),
            "Diagnostic codes missing documentation pages in docs/rules/: {:?}",
            missing
        );
    }

    #[test]
    fn test_summary_md_consistent() {
        let engine = build_doc_engine();
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
        let summary_path = workspace_root.join("docs/SUMMARY.md");

        let generated = engine.summary_md();
        agent_doc::assert_or_bless(&summary_path, &generated)
            .expect("SUMMARY.md is not up to date. Run with AGENT_DOC_BLESS=1 cargo test to update.");
    }

    #[test]
    fn test_skill_md_consistent() {
        let engine = build_doc_engine();
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
        let skill_path = workspace_root.join("skills/kinema/SKILL.md");

        let generated = engine.skill_md(
            "kinema-pcb-design",
            "Design, modify, route, and verify printed circuit boards (PCBs) in KiCad using kinema.",
        );
        agent_doc::assert_or_bless(&skill_path, &generated)
            .expect("SKILL.md is not up to date. Run with AGENT_DOC_BLESS=1 cargo test to update.");
    }

    #[test]
    fn test_llms_txt_consistent() {
        let engine = build_doc_engine();
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
        let llms_path = workspace_root.join("llms.txt");

        let generated = engine.llms_txt();
        agent_doc::assert_or_bless(&llms_path, &generated)
            .expect("llms.txt is not up to date. Run with AGENT_DOC_BLESS=1 cargo test to update.");
    }

    #[test]
    fn test_doc_links_valid() {
        let engine = build_doc_engine();
        let broken = engine.broken_links();
        assert!(
            broken.is_empty(),
            "Found broken internal markdown links in documentation:\n{:#?}",
            broken
        );
    }

    #[test]
    fn test_verilog_code_blocks_valid() {
        let engine = build_doc_engine();
        let blocks = engine.code_blocks("verilog");
        assert!(
            !blocks.is_empty(),
            "Expected at least one verilog code block in documentation"
        );

        for block in &blocks {
            if let Err(e) = kinema_syntax::parser::parse(&block.path, &block.source) {
                panic!(
                    "Failed to parse verilog code block in {}:{}: {:?}\nSource:\n{}",
                    block.path, block.line, e, block.source
                );
            }
        }
    }
}
