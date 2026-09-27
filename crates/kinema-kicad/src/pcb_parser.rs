use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PcbPad {
    pub pad_number: String,
    pub net_code: usize,
    pub net_name: String,
    pub pin_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PcbFootprint {
    pub footprint: String,
    pub refdes: String,
    pub value: String,
    pub mpn: Option<String>,
    pub dnp: bool,
    pub board_only: bool,
    pub tstamp: String,
    pub path: Option<String>,
    pub pads: Vec<PcbPad>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PcbBoard {
    pub nets: HashMap<usize, String>, // net_code -> net_name
    pub footprints: Vec<PcbFootprint>,
}

pub struct PcbParser<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> PcbParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            cursor: 0,
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.cursor < self.bytes.len() {
            let b = self.bytes[self.cursor];
            if b.is_ascii_whitespace() {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.cursor).copied()
    }

    fn advance_byte(&mut self) -> Option<u8> {
        if self.cursor < self.bytes.len() {
            let b = self.bytes[self.cursor];
            self.cursor += 1;
            Some(b)
        } else {
            None
        }
    }


    fn read_atom(&mut self) -> String {
        self.skip_whitespace_and_comments();
        if self.peek_byte() == Some(b'"') {
            self.advance_byte();
            let mut s = String::new();
            let mut escape = false;
            while let Some(b) = self.advance_byte() {
                if escape {
                    s.push(b as char);
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    break;
                } else {
                    s.push(b as char);
                }
            }
            s
        } else {
            let start = self.cursor;
            while let Some(b) = self.peek_byte() {
                if b.is_ascii_whitespace() || b == b'(' || b == b')' {
                    break;
                }
                self.advance_byte();
            }
            String::from_utf8_lossy(&self.bytes[start..self.cursor]).to_string()
        }
    }

    // Fast skip until the matching closing parenthesis of current node
    fn skip_current_list(&mut self) {
        let mut depth = 1;
        while self.cursor < self.bytes.len() && depth > 0 {
            let b = self.bytes[self.cursor];
            self.cursor += 1;
            if b == b'"' {
                // Inside quotes, skip until unescaped closing quote
                let mut escape = false;
                while self.cursor < self.bytes.len() {
                    let qb = self.bytes[self.cursor];
                    self.cursor += 1;
                    if escape {
                        escape = false;
                    } else if qb == b'\\' {
                        escape = true;
                    } else if qb == b'"' {
                        break;
                    }
                }
            } else if b == b'(' {
                depth += 1;
            } else if b == b')' {
                depth -= 1;
            }
        }
    }

    pub fn parse_board(&mut self) -> Result<PcbBoard, String> {
        self.skip_whitespace_and_comments();
        if self.advance_byte() != Some(b'(') {
            return Err("Expected '(' at start of file".into());
        }
        let root_tag = self.read_atom();
        if root_tag != "kicad_pcb" {
            return Err(format!("Expected 'kicad_pcb', found '{}'", root_tag));
        }

        let mut nets = HashMap::new();
        let mut footprints = Vec::new();

        while self.cursor < self.bytes.len() {
            self.skip_whitespace_and_comments();
            if self.peek_byte() == Some(b')') {
                self.advance_byte();
                break;
            }
            if self.peek_byte() != Some(b'(') {
                if self.peek_byte().is_none() {
                    break;
                }
                self.advance_byte();
                continue;
            }

            self.advance_byte(); // consume '('
            let tag = self.read_atom();

            match tag.as_str() {
                "net" => {
                    let code_str = self.read_atom();
                    let name = self.read_atom();
                    self.skip_current_list(); // close (net ...)
                    if let Ok(code) = code_str.parse::<usize>() {
                        nets.insert(code, name);
                    }
                }
                "footprint" | "module" => {
                    let fp = self.parse_footprint(&nets)?;
                    footprints.push(fp);
                }
                _ => {
                    // Fast skip zone, segment, via, arc, etc.
                    self.skip_current_list();
                }
            }
        }

        Ok(PcbBoard { nets, footprints })
    }

    fn parse_footprint(&mut self, net_lookup: &HashMap<usize, String>) -> Result<PcbFootprint, String> {
        let fp_name = self.read_atom();
        let mut refdes = String::new();
        let mut value = String::new();
        let mut mpn = None;
        let mut dnp = false;
        let mut board_only = false;
        let mut tstamp = String::new();
        let mut path = None;
        let mut pads = Vec::new();

        while self.cursor < self.bytes.len() {
            self.skip_whitespace_and_comments();
            if self.peek_byte() == Some(b')') {
                self.advance_byte();
                break;
            }
            if self.peek_byte() != Some(b'(') {
                self.advance_byte();
                continue;
            }

            self.advance_byte(); // consume '('
            let tag = self.read_atom();

            match tag.as_str() {
                "property" => {
                    let key = self.read_atom();
                    let val = self.read_atom();
                    self.skip_current_list();
                    match key.as_str() {
                        "Reference" => refdes = val,
                        "Value" => value = val,
                        "mpn" => mpn = Some(val),
                        "dnp" => dnp = true,
                        _ => {}
                    }
                }
                "fp_text" => {
                    let kind = self.read_atom();
                    let val = self.read_atom();
                    self.skip_current_list();
                    if kind == "reference" && refdes.is_empty() {
                        refdes = val;
                    } else if kind == "value" && value.is_empty() {
                        value = val;
                    }
                }
                "tstamp" => {
                    tstamp = self.read_atom();
                    self.skip_current_list();
                }
                "path" => {
                    path = Some(self.read_atom());
                    self.skip_current_list();
                }
                "attr" => {
                    let mut attr_tokens = Vec::new();
                    while self.peek_byte() != Some(b')') && self.cursor < self.bytes.len() {
                        attr_tokens.push(self.read_atom());
                    }
                    self.skip_current_list();
                    for a in attr_tokens {
                        if a == "board_only" {
                            board_only = true;
                        } else if a == "dnp" {
                            dnp = true;
                        }
                    }
                }
                "pad" => {
                    let pad_num = self.read_atom();
                    let pin_type = self.read_atom();
                    let mut net_code = 0;
                    let mut net_name = String::new();

                    // Parse inner pad properties looking for (net <code: usize> "<name: String>")
                    while self.cursor < self.bytes.len() {
                        self.skip_whitespace_and_comments();
                        if self.peek_byte() == Some(b')') {
                            self.advance_byte();
                            break;
                        }
                        if self.peek_byte() != Some(b'(') {
                            self.advance_byte();
                            continue;
                        }
                        self.advance_byte();
                        let sub_tag = self.read_atom();
                        if sub_tag == "net" {
                            let c_str = self.read_atom();
                            let n = self.read_atom();
                            self.skip_current_list();
                            if let Ok(c) = c_str.parse::<usize>() {
                                net_code = c;
                            }
                            net_name = if n.is_empty() {
                                net_lookup.get(&net_code).cloned().unwrap_or_default()
                            } else {
                                n
                            };
                        } else {
                            self.skip_current_list();
                        }
                    }

                    pads.push(PcbPad {
                        pad_number: pad_num,
                        net_code,
                        net_name,
                        pin_type,
                    });
                }
                _ => {
                    self.skip_current_list();
                }
            }
        }

        if tstamp.is_empty() {
            if let Some(p) = &path {
                tstamp = p.trim_start_matches('/').to_string();
            }
        }

        Ok(PcbFootprint {
            footprint: fp_name,
            refdes,
            value,
            mpn,
            dnp,
            board_only,
            tstamp,
            path,
            pads,
        })
    }
}

pub fn parse_kicad_pcb(content: &str) -> Result<PcbBoard, String> {
    let mut parser = PcbParser::new(content);
    parser.parse_board()
}
