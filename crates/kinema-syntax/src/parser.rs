use crate::ast::*;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{file}:{line}:{col}: {message}")]
pub struct SyntaxError {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    KwModule,
    KwEndmodule,
    KwInout,
    KwWire,
    KwParameter,
    Ident(String),
    StringLit(String),
    IntLit(u32),
    AttrOpen,  // (*
    AttrClose, // *)
    HashParen, // #(
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    LBrace,    // {
    RBrace,    // }
    Semi,      // ;
    Comma,     // ,
    Dot,       // .
    Colon,     // :
    Eq,        // =
    Comment {
        text: String,
        is_standalone: bool,
    },
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    file: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(file: &'a str, input: &'a str) -> Self {
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        Self {
            file,
            chars,
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|&(_, c)| c)
    }

    fn peek_offset(&self, offset: usize) -> Option<char> {
        self.chars.get(self.cursor + offset).map(|&(_, c)| c)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&(_, c)) = self.chars.get(self.cursor) {
            self.cursor += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn cur_pos(&self) -> (usize, usize, usize) {
        let byte_pos = self
            .chars
            .get(self.cursor)
            .map(|&(idx, _)| idx)
            .unwrap_or_else(|| {
                self.chars.last().map(|&(idx, c)| idx + c.len_utf8()).unwrap_or(0)
            });
        (byte_pos, self.line, self.col)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, SyntaxError> {
        let mut tokens = Vec::new();
        let mut line_has_tokens = false;
        let mut current_line = 1;

        while self.cursor < self.chars.len() {
            if self.line != current_line {
                current_line = self.line;
                line_has_tokens = false;
            }

            let (start_byte, start_line, start_col) = self.cur_pos();
            let c = match self.peek() {
                Some(c) => c,
                None => break,
            };

            // Whitespace (except newline which updates line tracking)
            if c.is_whitespace() {
                self.advance();
                continue;
            }

            // Disallowed comments /* ... */
            if c == '/' && self.peek_offset(1) == Some('*') {
                return Err(SyntaxError {
                    message: "Block comments '/*' are not allowed, only '//' line comments are permitted".to_string(),
                    file: self.file.to_string(),
                    line: start_line,
                    col: start_col,
                });
            }

            // Disallowed preprocessor `
            if c == '`' {
                return Err(SyntaxError {
                    message: "Preprocessor directives starting with '`' are not allowed".to_string(),
                    file: self.file.to_string(),
                    line: start_line,
                    col: start_col,
                });
            }

            // Line comment //
            if c == '/' && self.peek_offset(1) == Some('/') {
                self.advance();
                self.advance();
                let mut text = String::new();
                while let Some(ch) = self.peek() {
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                    text.push(ch);
                    self.advance();
                }
                let (end_byte, _, _) = self.cur_pos();
                let is_standalone = !line_has_tokens;
                tokens.push(Token {
                    kind: TokenKind::Comment {
                        text,
                        is_standalone,
                    },
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            // Compound tokens: (*, #()
            if c == '(' && self.peek_offset(1) == Some('*') {
                self.advance();
                self.advance();
                let (end_byte, _, _) = self.cur_pos();
                line_has_tokens = true;
                tokens.push(Token {
                    kind: TokenKind::AttrOpen,
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            if c == '*' && self.peek_offset(1) == Some(')') {
                self.advance();
                self.advance();
                let (end_byte, _, _) = self.cur_pos();
                line_has_tokens = true;
                tokens.push(Token {
                    kind: TokenKind::AttrClose,
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            if c == '#' && self.peek_offset(1) == Some('(') {
                self.advance();
                self.advance();
                let (end_byte, _, _) = self.cur_pos();
                line_has_tokens = true;
                tokens.push(Token {
                    kind: TokenKind::HashParen,
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            // Single char punctuation
            let single = match c {
                '(' => Some(TokenKind::LParen),
                ')' => Some(TokenKind::RParen),
                '[' => Some(TokenKind::LBracket),
                ']' => Some(TokenKind::RBracket),
                '{' => Some(TokenKind::LBrace),
                '}' => Some(TokenKind::RBrace),
                ';' => Some(TokenKind::Semi),
                ',' => Some(TokenKind::Comma),
                '.' => Some(TokenKind::Dot),
                ':' => Some(TokenKind::Colon),
                '=' => Some(TokenKind::Eq),
                _ => None,
            };

            if let Some(kind) = single {
                self.advance();
                let (end_byte, _, _) = self.cur_pos();
                line_has_tokens = true;
                tokens.push(Token {
                    kind,
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            // String literals: "..."
            if c == '"' {
                self.advance(); // consume opening "
                let mut s = String::new();
                let mut closed = false;
                while let Some(ch) = self.advance() {
                    if ch == '"' {
                        closed = true;
                        break;
                    } else if ch == '\\' {
                        if let Some(next_ch) = self.advance() {
                            if next_ch == '"' || next_ch == '\\' {
                                s.push(next_ch);
                            } else {
                                return Err(SyntaxError {
                                    message: format!("Invalid escape sequence '\\{}', only '\\\"' and '\\\\' are permitted in strings", next_ch),
                                    file: self.file.to_string(),
                                    line: start_line,
                                    col: start_col,
                                });
                            }
                        } else {
                            return Err(SyntaxError {
                                message: "Unterminated string literal".to_string(),
                                file: self.file.to_string(),
                                line: start_line,
                                col: start_col,
                            });
                        }
                    } else if ch == '\n' || ch == '\r' {
                        return Err(SyntaxError {
                            message: "Newline inside string literal is not allowed".to_string(),
                            file: self.file.to_string(),
                            line: start_line,
                            col: start_col,
                        });
                    } else {
                        s.push(ch);
                    }
                }
                if !closed {
                    return Err(SyntaxError {
                        message: "Unterminated string literal".to_string(),
                        file: self.file.to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                let (end_byte, _, _) = self.cur_pos();
                line_has_tokens = true;
                tokens.push(Token {
                    kind: TokenKind::StringLit(s),
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            // Numbers: non-negative integers
            if c.is_ascii_digit() {
                let mut num_str = String::new();
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val: u32 = num_str.parse().map_err(|_| SyntaxError {
                    message: format!("Integer literal overflow: {}", num_str),
                    file: self.file.to_string(),
                    line: start_line,
                    col: start_col,
                })?;
                let (end_byte, _, _) = self.cur_pos();
                line_has_tokens = true;
                tokens.push(Token {
                    kind: TokenKind::IntLit(val),
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            // Identifiers or keywords: [A-Za-z_][A-Za-z0-9_]*
            if c.is_ascii_alphabetic() || c == '_' {
                let mut ident = String::new();
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let (end_byte, _, _) = self.cur_pos();
                let kind = match ident.as_str() {
                    "module" => TokenKind::KwModule,
                    "endmodule" => TokenKind::KwEndmodule,
                    "inout" => TokenKind::KwInout,
                    "wire" => TokenKind::KwWire,
                    "parameter" => TokenKind::KwParameter,
                    _ => TokenKind::Ident(ident),
                };
                line_has_tokens = true;
                tokens.push(Token {
                    kind,
                    span: Span {
                        file: self.file.to_string(),
                        start: start_byte,
                        end: end_byte,
                        line: start_line,
                        col: start_col,
                    },
                });
                continue;
            }

            return Err(SyntaxError {
                message: format!("Unexpected character '{}'", c),
                file: self.file.to_string(),
                line: start_line,
                col: start_col,
            });
        }

        let (end_byte, end_line, end_col) = self.cur_pos();
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                file: self.file.to_string(),
                start: end_byte,
                end: end_byte,
                line: end_line,
                col: end_col,
            },
        });

        Ok(tokens)
    }
}

pub struct Parser<'a> {
    file: &'a str,
    tokens: Vec<Token>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(file: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            file,
            tokens,
            cursor: 0,
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.cursor].clone();
        if self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
        tok
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if &self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind, desc: &str) -> Result<Token, SyntaxError> {
        let cur = self.peek();
        if &cur.kind == kind {
            Ok(self.advance())
        } else {
            Err(SyntaxError {
                message: format!("Expected {}, found {:?}", desc, cur.kind),
                file: self.file.to_string(),
                line: cur.span.line,
                col: cur.span.col,
            })
        }
    }

    fn expect_ident(&mut self, desc: &str) -> Result<(String, Span), SyntaxError> {
        let cur = self.peek();
        match &cur.kind {
            TokenKind::Ident(s) => {
                let s = s.clone();
                let span = cur.span.clone();
                self.advance();
                Ok((s, span))
            }
            _ => Err(SyntaxError {
                message: format!("Expected {}, found {:?}", desc, cur.kind),
                file: self.file.to_string(),
                line: cur.span.line,
                col: cur.span.col,
            }),
        }
    }

    fn expect_string_lit(&mut self, desc: &str) -> Result<(String, Span), SyntaxError> {
        let cur = self.peek();
        match &cur.kind {
            TokenKind::StringLit(s) => {
                let s = s.clone();
                let span = cur.span.clone();
                self.advance();
                Ok((s, span))
            }
            _ => Err(SyntaxError {
                message: format!("Expected {}, found {:?}", desc, cur.kind),
                file: self.file.to_string(),
                line: cur.span.line,
                col: cur.span.col,
            }),
        }
    }

    fn expect_int_lit(&mut self, desc: &str) -> Result<(u32, Span), SyntaxError> {
        let cur = self.peek();
        match &cur.kind {
            TokenKind::IntLit(n) => {
                let n = *n;
                let span = cur.span.clone();
                self.advance();
                Ok((n, span))
            }
            _ => Err(SyntaxError {
                message: format!("Expected {}, found {:?}", desc, cur.kind),
                file: self.file.to_string(),
                line: cur.span.line,
                col: cur.span.col,
            }),
        }
    }

    fn collect_comments(&mut self) -> (Vec<String>, Option<String>) {
        let mut leading = Vec::new();
        let mut trailing = None;

        while let TokenKind::Comment { text, is_standalone } = &self.peek().kind {
            if *is_standalone {
                leading.push(text.clone());
            } else if trailing.is_none() {
                trailing = Some(text.clone());
            } else {
                leading.push(text.clone());
            }
            self.advance();
        }

        (leading, trailing)
    }

    pub fn parse_file(&mut self) -> Result<SourceFile, SyntaxError> {
        let mut modules = Vec::new();

        while self.peek().kind != TokenKind::Eof {
            let (leading_comments, _) = self.collect_comments();
            if self.peek().kind == TokenKind::Eof {
                return Ok(SourceFile {
                    modules,
                    eof_comments: leading_comments,
                });
            }
            let module = self.parse_module(leading_comments)?;
            modules.push(module);
        }

        let (eof_comments, _) = self.collect_comments();

        Ok(SourceFile {
            modules,
            eof_comments,
        })
    }

    fn parse_attrs(&mut self) -> Result<Vec<Attr>, SyntaxError> {
        let mut attrs = Vec::new();
        while self.peek().kind == TokenKind::AttrOpen {
            self.advance(); // consume (*
            loop {
                let (key, key_span) = self.expect_ident("attribute identifier")?;
                let mut val = None;
                let mut end_span = key_span.clone();

                if self.match_token(&TokenKind::Eq) {
                    let (s, s_span) = self.expect_string_lit("string literal")?;
                    end_span = s_span;
                    val = Some(s);
                }

                attrs.push(Attr {
                    key,
                    value: val,
                    span: Span {
                        file: self.file.to_string(),
                        start: key_span.start,
                        end: end_span.end,
                        line: key_span.line,
                        col: key_span.col,
                    },
                });

                if self.match_token(&TokenKind::Comma) {
                    continue;
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::AttrClose, "'*)'")?;
        }
        Ok(attrs)
    }

    fn parse_range(&mut self) -> Result<RangeDef, SyntaxError> {
        let open_tok = self.expect(&TokenKind::LBracket, "'['")?;
        let (msb, _) = self.expect_int_lit("msb integer")?;
        self.expect(&TokenKind::Colon, "':'")?;
        let (lsb, lsb_span) = self.expect_int_lit("integer literal '0'")?;
        if lsb != 0 {
            return Err(SyntaxError {
                message: format!("Range must end in 0 per spec [INT:0], found {}", lsb),
                file: self.file.to_string(),
                line: lsb_span.line,
                col: lsb_span.col,
            });
        }
        let close_tok = self.expect(&TokenKind::RBracket, "']'")?;
        Ok(RangeDef {
            msb,
            lsb: 0,
            span: Span {
                file: self.file.to_string(),
                start: open_tok.span.start,
                end: close_tok.span.end,
                line: open_tok.span.line,
                col: open_tok.span.col,
            },
        })
    }

    fn parse_module(&mut self, leading_comments: Vec<String>) -> Result<ModuleDef, SyntaxError> {
        let attrs = self.parse_attrs()?;
        let (extra_leading, _) = self.collect_comments();
        let mut combined_leading = leading_comments;
        combined_leading.extend(extra_leading);

        let mod_tok = self.expect(&TokenKind::KwModule, "'module'")?;
        let (mod_name, _) = self.expect_ident("module identifier")?;

        // Optional parameters: #(parameter ident = "str", ...)
        let mut params = Vec::new();
        if self.match_token(&TokenKind::HashParen) {
            loop {
                self.expect(&TokenKind::KwParameter, "'parameter'")?;
                let (pname, pspan) = self.expect_ident("parameter identifier")?;
                self.expect(&TokenKind::Eq, "'='")?;
                let (val, vspan) = self.expect_string_lit("string literal")?;

                params.push(ParamDef {
                    name: pname,
                    value: val,
                    span: Span {
                        file: self.file.to_string(),
                        start: pspan.start,
                        end: vspan.end,
                        line: pspan.line,
                        col: pspan.col,
                    },
                });

                if self.match_token(&TokenKind::Comma) {
                    continue;
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RParen, "')'")?;
        }

        // Ports: (port, port, ...)
        self.expect(&TokenKind::LParen, "'('")?;
        let mut ports = Vec::new();
        if self.peek().kind != TokenKind::RParen {
            loop {
                let (port_leading, _) = self.collect_comments();
                let port_attrs = self.parse_attrs()?;
                let inout_tok = self.expect(&TokenKind::KwInout, "'inout'")?;
                let range = if self.peek().kind == TokenKind::LBracket {
                    Some(self.parse_range()?)
                } else {
                    None
                };
                let (port_name, port_span) = self.expect_ident("port identifier")?;

                let trailing_comment = if let TokenKind::Comment { text, is_standalone: false } = &self.peek().kind {
                    let comment = text.clone();
                    self.advance();
                    Some(comment)
                } else {
                    None
                };

                ports.push(PortDef {
                    leading_comments: port_leading,
                    attrs: port_attrs,
                    range,
                    name: port_name,
                    trailing_comment,
                    span: Span {
                        file: self.file.to_string(),
                        start: inout_tok.span.start,
                        end: port_span.end,
                        line: inout_tok.span.line,
                        col: inout_tok.span.col,
                    },
                });

                if self.match_token(&TokenKind::Comma) {
                    continue;
                } else {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen, "')'")?;
        self.expect(&TokenKind::Semi, "';'")?;

        // Items: wire_decl | instance
        let mut items = Vec::new();

        let endmodule_comments = loop {
            let (item_leading, _) = self.collect_comments();
            if self.peek().kind == TokenKind::KwEndmodule {
                break item_leading;
            }

            let item_attrs = self.parse_attrs()?;
            let (more_leading, _) = self.collect_comments();
            let mut all_item_leading = item_leading;
            all_item_leading.extend(more_leading);

            if self.match_token(&TokenKind::KwWire) {
                // wire_decl = { attr } "wire" [ range ] IDENT ";" ;
                let range = if self.peek().kind == TokenKind::LBracket {
                    Some(self.parse_range()?)
                } else {
                    None
                };
                let (wire_name, wire_span) = self.expect_ident("wire identifier")?;
                self.expect(&TokenKind::Semi, "';'")?;
                let trailing_comment = if let TokenKind::Comment { text, is_standalone: false } = &self.peek().kind {
                    let comment = text.clone();
                    self.advance();
                    Some(comment)
                } else {
                    None
                };

                items.push(Item::Wire(WireDecl {
                    leading_comments: all_item_leading,
                    attrs: item_attrs,
                    range,
                    name: wire_name,
                    trailing_comment,
                    span: wire_span,
                }));
            } else {
                // instance = { attr } IDENT [ param_ovr ] IDENT "(" [ conn { "," conn } ] ")" ";" ;
                let (target_mod_name, _) = self.expect_ident("module or leaf identifier")?;

                // param_ovr = "#(" "." IDENT "(" STRING ")" { "," "." IDENT "(" STRING ")" } ")" ;
                let mut param_overrides = Vec::new();
                if self.match_token(&TokenKind::HashParen) {
                    loop {
                        self.expect(&TokenKind::Dot, "'.' in parameter override")?;
                        let (p_name, p_span) = self.expect_ident("parameter identifier")?;
                        self.expect(&TokenKind::LParen, "'('")?;
                        let (val, val_span) = self.expect_string_lit("string literal")?;
                        self.expect(&TokenKind::RParen, "')'")?;
                        param_overrides.push(ParamOverride {
                            name: p_name,
                            value: val,
                            span: Span {
                                file: self.file.to_string(),
                                start: p_span.start,
                                end: val_span.end,
                                line: p_span.line,
                                col: p_span.col,
                            },
                        });

                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen, "')'")?;
                }

                let (inst_name, inst_span) = self.expect_ident("instance identifier")?;

                self.expect(&TokenKind::LParen, "'('")?;
                let mut conns = Vec::new();
                if self.peek().kind != TokenKind::RParen {
                    loop {
                        self.expect(&TokenKind::Dot, "'.' in port connection")?;
                        let (p_name, p_span) = self.expect_ident("port identifier")?;
                        self.expect(&TokenKind::LParen, "'('")?;
                        let expr = if self.peek().kind != TokenKind::RParen {
                            Some(self.parse_expr()?)
                        } else {
                            None
                        };
                        self.expect(&TokenKind::RParen, "')'")?;

                        let conn_trailing = if let TokenKind::Comment { text, is_standalone: false } = &self.peek().kind {
                            let comment = text.clone();
                            self.advance();
                            Some(comment)
                        } else {
                            None
                        };

                        conns.push(PortConnection {
                            port_name: p_name,
                            expr,
                            trailing_comment: conn_trailing,
                            span: p_span,
                        });

                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RParen, "')'")?;
                self.expect(&TokenKind::Semi, "';'")?;

                let inst_trailing = if let TokenKind::Comment { text, is_standalone: false } = &self.peek().kind {
                    let comment = text.clone();
                    self.advance();
                    Some(comment)
                } else {
                    None
                };

                items.push(Item::Instance(Instance {
                    leading_comments: all_item_leading,
                    attrs: item_attrs,
                    module_name: target_mod_name,
                    param_overrides,
                    instance_name: inst_name,
                    connections: conns,
                    trailing_comment: inst_trailing,
                    span: inst_span,
                }));
            }
        };

        let end_tok = self.expect(&TokenKind::KwEndmodule, "'endmodule'")?;

        Ok(ModuleDef {
            leading_comments: combined_leading,
            attrs,
            name: mod_name,
            params,
            ports,
            items,
            endmodule_comments,
            span: Span {
                file: self.file.to_string(),
                start: mod_tok.span.start,
                end: end_tok.span.end,
                line: mod_tok.span.line,
                col: mod_tok.span.col,
            },
        })
    }

    fn parse_ref(&mut self) -> Result<RefExpr, SyntaxError> {
        let (ident, ident_span) = self.expect_ident("identifier")?;

        let mut index = None;
        let mut end_span = ident_span.clone();
        if self.match_token(&TokenKind::LBracket) {
            let (int1, _) = self.expect_int_lit("integer literal")?;
            if self.match_token(&TokenKind::Colon) {
                let (int2, int2_span) = self.expect_int_lit("integer literal")?;
                let close_tok = self.expect(&TokenKind::RBracket, "']'")?;
                end_span = close_tok.span;
                index = Some(RefIndex::Range(int1, int2));
                let _ = int2_span;
            } else {
                let close_tok = self.expect(&TokenKind::RBracket, "']'")?;
                end_span = close_tok.span;
                index = Some(RefIndex::Single(int1));
            }
        }

        Ok(RefExpr {
            ident,
            index,
            span: Span {
                file: self.file.to_string(),
                start: ident_span.start,
                end: end_span.end,
                line: ident_span.line,
                col: ident_span.col,
            },
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, SyntaxError> {
        if self.match_token(&TokenKind::LBrace) {
            let mut refs = Vec::new();
            loop {
                refs.push(self.parse_ref()?);
                if self.match_token(&TokenKind::Comma) {
                    continue;
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RBrace, "'}'")?;
            Ok(Expr::Concat(refs))
        } else {
            let r = self.parse_ref()?;
            Ok(Expr::Ref(r))
        }
    }
}

pub fn parse(file: &str, input: &str) -> Result<SourceFile, SyntaxError> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut lexer = Lexer::new(file, input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(file, tokens);
    parser.parse_file()
}
