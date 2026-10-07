pub mod index;
pub mod model;
pub mod parser;
pub mod renderer;

pub use index::DocIndex;
pub use model::{DocPage, DocSection, RenderMode, ResolvedLink, TopicSummary};
pub use parser::parse_markdown_page;
pub use renderer::DocRenderer;

#[derive(Debug, thiserror::Error)]
pub enum DocError {
    #[error("Topic '{0}' not found in documentation index")]
    NotFound(String),
}

#[derive(Debug, Clone)]
pub struct DocEngine {
    pub binary_name: String,
    pub guide_subcommand: String,
    pub explain_subcommand: String,
    pub index: DocIndex,
}

impl DocEngine {
    pub fn new(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
            guide_subcommand: "guide".to_string(),
            explain_subcommand: "explain".to_string(),
            index: DocIndex::new(),
        }
    }

    pub fn with_subcommands(
        mut self,
        guide_subcommand: impl Into<String>,
        explain_subcommand: impl Into<String>,
    ) -> Self {
        self.guide_subcommand = guide_subcommand.into();
        self.explain_subcommand = explain_subcommand.into();
        self
    }

    pub fn add_page(&mut self, path: &str, content: &str) {
        let page = parse_markdown_page(path, content);
        self.index.add_page(page);
    }

    pub fn add_pages<'a, I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        for (path, content) in pages {
            self.add_page(path, content);
        }
    }

    pub fn lookup(&self, query: &str) -> Option<&DocPage> {
        self.index.lookup(query)
    }

    pub fn list_topics(&self, category: Option<&str>) -> Vec<TopicSummary> {
        self.index.list_topics(category)
    }

    pub fn explain(&self, query: &str, mode: RenderMode) -> Result<String, DocError> {
        if let Some(page) = self.index.lookup(query) {
            let renderer = DocRenderer::new(
                &self.binary_name,
                &self.guide_subcommand,
                &self.explain_subcommand,
                &self.index,
            );
            Ok(renderer.render_page(page, mode))
        } else {
            Err(DocError::NotFound(query.to_string()))
        }
    }

    pub fn guide(&self, topic_opt: Option<&str>, mode: RenderMode) -> Result<String, DocError> {
        if let Some(topic) = topic_opt {
            self.explain(topic, mode)
        } else {
            // Render index overview
            let mut out = String::new();
            let topics = self.list_topics(None);

            if mode == RenderMode::Terminal {
                out.push_str(&format!(
                    "\x1b[1;36m=== {} Documentation & Topic Index ===\x1b[0m\n\n",
                    self.binary_name
                ));
            } else {
                out.push_str(&format!(
                    "# {} Documentation & Topic Index\n\n",
                    self.binary_name
                ));
            }

            let mut guides = Vec::new();
            let mut rules = Vec::new();
            let mut others = Vec::new();

            for t in topics {
                match t.category.as_str() {
                    "guide" => guides.push(t),
                    "rule" => rules.push(t),
                    _ => others.push(t),
                }
            }

            if !guides.is_empty() {
                out.push_str("## Guides & Best Practices\n");
                for g in guides {
                    let cmd = format!("{} {} {}", self.binary_name, self.guide_subcommand, g.id);
                    if mode == RenderMode::Terminal {
                        out.push_str(&format!("  * \x1b[1m{}\x1b[0m: {} \x1b[90m(`{}`)\x1b[0m\n", g.id, g.summary, cmd));
                    } else {
                        out.push_str(&format!("  * **{}**: {} (`{}`)\n", g.id, g.summary, cmd));
                    }
                }
                out.push('\n');
            }

            if !rules.is_empty() {
                out.push_str("## Diagnostics & Rules\n");
                for r in rules {
                    let cmd = format!("{} {} {}", self.binary_name, self.explain_subcommand, r.id);
                    if mode == RenderMode::Terminal {
                        out.push_str(&format!("  * \x1b[1m{}\x1b[0m: {} \x1b[90m(`{}`)\x1b[0m\n", r.id, r.summary, cmd));
                    } else {
                        out.push_str(&format!("  * **{}**: {} (`{}`)\n", r.id, r.summary, cmd));
                    }
                }
                out.push('\n');
            }

            if !others.is_empty() {
                out.push_str("## Other Topics\n");
                for o in others {
                    let cmd = format!("{} {} {}", self.binary_name, self.guide_subcommand, o.id);
                    out.push_str(&format!("  * **{}**: {} (`{}`)\n", o.id, o.summary, cmd));
                }
                out.push('\n');
            }

            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_doc_index_and_link_rewriting() {
        let mut engine = DocEngine::new("kinema");

        let guide_md = r#"---
id: identity
title: Component Identity and UUID Management
aliases: [uuid, refactoring]
summary: How to keep PCB footprint placement intact when renaming components.
---

# Component Identity

KiCad identifies parts by UUID v5.
See [Rule: identity-missing](../rules/identity-missing.md) for related diagnostics.
"#;

        let rule_md = r#"---
id: identity-missing
title: Missing or Mismatched Footprint Identity
aliases: [uuid-mismatch]
summary: PCB footprint timestamp does not match circuit UUID v5.
---

# Missing Identity

When refdes changes, check [Identity Guide](../guides/identity.md).
Also check external docs at [KiCad Docs](https://docs.kicad.org).
"#;

        engine.add_page("guides/identity.md", guide_md);
        engine.add_page("rules/identity-missing.md", rule_md);

        // Test lookup
        let page = engine.lookup("identity").expect("lookup identity");
        assert_eq!(page.id, "identity");

        let page_alias = engine.lookup("uuid-mismatch").expect("lookup alias");
        assert_eq!(page_alias.id, "identity-missing");

        // Test agent mode rendering with link rewriting to CLI commands
        let rendered_rule = engine
            .explain("identity-missing", RenderMode::Agent)
            .expect("render explain");

        assert!(
            rendered_rule.contains("Identity Guide (see: 'kinema guide identity')"),
            "Internal link must be rewritten to executable guide command: {}",
            rendered_rule
        );
        assert!(
            rendered_rule.contains("KiCad Docs (https://docs.kicad.org)"),
            "External link must preserve URL: {}",
            rendered_rule
        );

        // Test guide mode rendering
        let rendered_guide = engine
            .explain("identity", RenderMode::Agent)
            .expect("render guide");

        assert!(
            rendered_guide.contains("Rule: identity-missing (see: 'kinema explain identity-missing')"),
            "Rule link must be rewritten to executable explain command: {}",
            rendered_guide
        );

        // Test terminal mode rendering
        let rendered_term = engine
            .explain("identity-missing", RenderMode::Terminal)
            .expect("render terminal");
        assert!(rendered_term.contains("\x1b]8;;https://docs.kicad.org\x1b\\"), "OSC 8 link expected for external URL");

        // Test overview listing
        let overview = engine.guide(None, RenderMode::Agent).expect("render overview");
        assert!(overview.contains("## Guides & Best Practices"));
        assert!(overview.contains("## Diagnostics & Rules"));
        assert!(overview.contains("`kinema guide identity`"));
        assert!(overview.contains("`kinema explain identity-missing`"));
    }
}
