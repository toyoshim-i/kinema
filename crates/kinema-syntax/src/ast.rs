use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub file: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn dummy() -> Self {
        Self {
            file: String::new(),
            start: 0,
            end: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn to_location(&self) -> SourceLocation {
        SourceLocation {
            file: self.file.clone(),
            line: self.line,
            col: self.col,
        }
    }
}

pub const CANONICAL_ATTR_ORDER: &[&str] = &[
    "footprint",
    "mpn",
    "prefix",
    "pad",
    "etype",
    "decouple",
    "id",
    "ref",
    "dnp",
    "nearby",
    "width",
    "current",
    "netclass",
    "diffpair",
];

pub fn attr_canonical_index(key: &str) -> usize {
    CANONICAL_ATTR_ORDER
        .iter()
        .position(|&k| k == key)
        .unwrap_or(usize::MAX)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attr {
    pub key: String,
    pub value: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeDef {
    pub msb: u32,
    pub lsb: u32,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortDef {
    pub leading_comments: Vec<String>,
    pub attrs: Vec<Attr>,
    pub range: Option<RangeDef>,
    pub name: String,
    pub trailing_comment: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireDecl {
    pub leading_comments: Vec<String>,
    pub attrs: Vec<Attr>,
    pub range: Option<RangeDef>,
    pub name: String,
    pub trailing_comment: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefIndex {
    Single(u32),
    Range(u32, u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefExpr {
    pub ident: String,
    pub index: Option<RefIndex>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Ref(RefExpr),
    Concat(Vec<RefExpr>),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ref(r) => r.span.clone(),
            Expr::Concat(list) => {
                if let (Some(first), Some(last)) = (list.first(), list.last()) {
                    Span {
                        file: first.span.file.clone(),
                        start: first.span.start,
                        end: last.span.end,
                        line: first.span.line,
                        col: first.span.col,
                    }
                } else {
                    Span::dummy()
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamOverride {
    pub name: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortConnection {
    pub port_name: String,
    pub expr: Option<Expr>,
    pub trailing_comment: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    pub leading_comments: Vec<String>,
    pub attrs: Vec<Attr>,
    pub module_name: String,
    pub param_overrides: Vec<ParamOverride>,
    pub instance_name: String,
    pub connections: Vec<PortConnection>,
    pub trailing_comment: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    Wire(WireDecl),
    Instance(Instance),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Wire(w) => w.span.clone(),
            Item::Instance(i) => i.span.clone(),
        }
    }

    pub fn leading_comments(&self) -> &[String] {
        match self {
            Item::Wire(w) => &w.leading_comments,
            Item::Instance(i) => &i.leading_comments,
        }
    }

    pub fn is_wire(&self) -> bool {
        matches!(self, Item::Wire(_))
    }

    pub fn is_instance(&self) -> bool {
        matches!(self, Item::Instance(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDef {
    pub leading_comments: Vec<String>,
    pub attrs: Vec<Attr>,
    pub name: String,
    pub params: Vec<ParamDef>,
    pub ports: Vec<PortDef>,
    pub items: Vec<Item>,
    pub endmodule_comments: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub modules: Vec<ModuleDef>,
    pub eof_comments: Vec<String>,
}
