use agent_doc::DocEngine;

pub fn build_doc_engine() -> DocEngine {
    let mut engine = DocEngine::new("kinema")
        .with_subcommands("guide", "explain");

    engine.add_page("guides/agent.md", include_str!("../../../docs/guides/agent.md"));
    engine.add_page("guides/workflow.md", include_str!("../../../docs/guides/workflow.md"));
    engine.add_page("guides/syntax.md", include_str!("../../../docs/guides/syntax.md"));
    engine.add_page("guides/identity.md", include_str!("../../../docs/guides/identity.md"));
    engine.add_page("guides/connection.md", include_str!("../../../docs/guides/connection.md"));
    engine.add_page("rules/identity-missing.md", include_str!("../../../docs/rules/identity-missing.md"));
    engine.add_page("rules/decouple-missing.md", include_str!("../../../docs/rules/decouple-missing.md"));

    engine
}
