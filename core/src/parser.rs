// Copyright (C) 2019 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::str::FromStr;

use crate::colour::Colour;
use crate::error::{Error, Result};
use crate::gene::Gene;
use crate::keywords::Keyword;
use crate::lexer::{tokenize, Token};
use crate::name::Name;
use crate::native::Native;

use strum::IntoEnumIterator;

#[derive(Clone, Debug, PartialEq)]
pub struct NodeMeta {
    pub gene: Option<Gene>,
    pub parameter_ast: Vec<Node>,
    pub parameter_prefix: Vec<Node>, // todo: couldn't this just be a String?
}

impl NodeMeta {
    pub fn new_with_gene(gene: Gene) -> Self {
        NodeMeta {
            gene: Some(gene),
            parameter_ast: Vec::new(),
            parameter_prefix: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    List(Vec<Node>, Option<NodeMeta>),
    Vector(Vec<Node>, Option<NodeMeta>),
    Float(f32, String, Option<NodeMeta>),
    Name(String, Name, Option<NodeMeta>),  // text, iname, meta
    Label(String, Name, Option<NodeMeta>), // text, iname, meta
    String(String, Option<NodeMeta>),
    Whitespace(String, Option<NodeMeta>),
    Comment(String, Option<NodeMeta>),
}

impl Node {
    pub fn is_semantic(&self) -> bool {
        match *self {
            Node::Comment(_, _) | Node::Whitespace(_, _) => false,
            _ => true,
        }
    }

    pub fn get_float(&self, use_genes: bool) -> Result<f32> {
        if let Node::Float(f, _, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            Gene::Float(f) => return Ok(*f),
                            _ => {
                                return Err(Error::Compiler(
                                    "Node::get_float incompatible gene".to_string(),
                                ));
                            }
                        }
                    }
                }
            } else {
                return Ok(*f);
            }
        }
        Err(Error::Compiler(format!(
            "Node::get_float expected Node::Float not {:?}",
            self
        )))
    }

    pub fn get_iname(&self, use_genes: bool) -> Result<Name> {
        if let Node::Name(_text, iname, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            Gene::Name(i) => return Ok(*i),
                            _ => {
                                return Err(Error::Compiler(
                                    "Node::get_iname incompatible gene".to_string(),
                                ));
                            }
                        }
                    }
                }
            } else {
                return Ok(*iname);
            }
        }
        Err(Error::Compiler(format!(
            "Node::get_iname expected Node::Name not {:?}",
            self
        )))
    }

    pub fn get_label_iname(&self, use_genes: bool) -> Result<Name> {
        if let Node::Label(_, iname, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            // todo: what type of gene would a Node::Label have?
                            Gene::Int(i) => return Ok(Name::new(*i)),
                            Gene::Name(i) => return Ok(*i),
                            _ => {
                                return Err(Error::Compiler(
                                    "Node::get_label_iname incompatible gene".to_string(),
                                ));
                            }
                        }
                    }
                }
            } else {
                return Ok(*iname);
            }
        }
        Err(Error::Compiler(format!(
            "Node::get_label_iname expected Node::Label not {:?}",
            self
        )))
    }

    pub fn get_colour(&self, use_genes: bool) -> Result<Colour> {
        if let Node::List(_, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            Gene::Colour(col) => return Ok(*col),
                            _ => {
                                return Err(Error::Compiler(
                                    "Node::get_colour incompatible gene".to_string(),
                                ));
                            }
                        }
                    }
                }
            } else {
                return Err(Error::Compiler(
                    "Node::get_colour expected to use gene".to_string(),
                ));
            }
        }
        Err(Error::Compiler(format!(
            "Node::get_colour expected Node::List not {:?}",
            self
        )))
    }

    pub fn get_2d(&self, use_genes: bool) -> Result<(f32, f32)> {
        if let Node::Vector(_, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            Gene::V2D(x, y) => return Ok((*x, *y)),
                            _ => {
                                return Err(Error::Compiler(
                                    "Node::get_2d incompatible gene".to_string(),
                                ));
                            }
                        }
                    }
                }
            } else {
                return Err(Error::Compiler(
                    "Node::get_2d expected to use gene".to_string(),
                ));
            }
        }
        Err(Error::Compiler(format!(
            "Node::get_2d expected Node::Vector not {:?}",
            self
        )))
    }

    pub fn is_alterable(&self) -> bool {
        match self {
            Node::List(_, meta)
            | Node::Vector(_, meta)
            | Node::Float(_, _, meta)
            | Node::Name(_, _, meta)
            | Node::Label(_, _, meta)
            | Node::String(_, meta)
            | Node::Whitespace(_, meta)
            | Node::Comment(_, meta) => meta.is_some(),
        }
    }

    pub fn has_gene(&self) -> bool {
        match self {
            Node::List(_, meta)
            | Node::Vector(_, meta)
            | Node::Float(_, _, meta)
            | Node::Name(_, _, meta)
            | Node::Label(_, _, meta)
            | Node::String(_, meta)
            | Node::Whitespace(_, meta)
            | Node::Comment(_, meta) => {
                if let Some(meta) = meta {
                    return meta.gene.is_some();
                } else {
                    false
                }
            }
        }
    }
}

struct NodeAndRemainder<'a> {
    node: Node,
    tokens: &'a [Token<'a>],
}

#[derive(Default, Debug)]
pub struct WordLut {
    // requires a builtin hashmap (function names reserved by the builtin api)
    // a keyword hashmap (keywords + constants + common arguments to builtin api functions)
    // a word hashmap (user defined names and labels)
    word_to_iname: HashMap<String, Name>,
    word_count: i32,

    iname_to_word: HashMap<Name, String>,
    iname_to_native: HashMap<Name, String>,
    iname_to_keyword: HashMap<Name, String>,
}

impl WordLut {
    pub fn new() -> WordLut {
        // native
        let mut n: HashMap<Name, String> = HashMap::new();
        for nat in Native::iter() {
            n.insert(Name::from(nat), nat.to_string());
        }

        // keyword
        let mut k: HashMap<Name, String> = HashMap::new();
        for kw in Keyword::iter() {
            k.insert(Name::from(kw), kw.to_string());
        }

        WordLut {
            word_to_iname: HashMap::new(),
            word_count: 0,

            iname_to_word: HashMap::new(),
            iname_to_native: n,
            iname_to_keyword: k,
        }
    }

    pub fn get_string_from_name(&self, name: Name) -> Option<&String> {
        if let Some(s) = self.iname_to_native.get(&name) {
            // 1st check the native api
            Some(s)
        } else if let Some(s) = self.iname_to_keyword.get(&name) {
            // 2nd check the keywords
            Some(s)
        } else {
            // finally check the iname_to_word
            self.iname_to_word.get(&name)
        }
    }

    fn get_or_add(&mut self, s: &str) -> Name {
        if let Some(i) = self.get_name_from_string(s) {
            return i;
        }

        let name = Name::new(self.word_count);
        self.word_to_iname.insert(s.to_string(), name);
        self.iname_to_word.insert(name, s.to_string());
        self.word_count += 1;

        Name::new(self.word_count - 1)
    }

    fn get_name_from_string(&self, s: &str) -> Option<Name> {
        // 1st check the native api
        if let Ok(n) = Native::from_str(s) {
            return Some(Name::from(n));
        }

        // 2nd check the keywords
        if let Ok(kw) = Keyword::from_str(s) {
            return Some(Name::from(kw));
        }

        // finally check/add to word_to_iname
        if let Some(i) = self.word_to_iname.get(s) {
            return Some(*i);
        }

        None
    }
}

pub fn parse(s: &str) -> Result<(Vec<Node>, WordLut)> {
    let t = tokenize(s)?;

    let mut tokens = t.as_slice();
    let mut res = Vec::new();

    let mut word_lut = WordLut::new();

    while !tokens.is_empty() {
        match eat_token(tokens, None, &mut word_lut) {
            Ok(nar) => {
                res.push(nar.node);
                tokens = nar.tokens;
            }
            Err(e) => return Err(e),
        }
    }

    Ok((res, word_lut))
}

// At the first token after a Token::ParenStart
//
fn eat_list<'a>(
    t: &'a [Token<'a>],
    meta: Option<NodeMeta>,
    word_lut: &mut WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    loop {
        match tokens[0] {
            Token::ParenEnd => {
                return Ok(NodeAndRemainder {
                    node: Node::List(res, meta),
                    tokens: &tokens[1..],
                });
            }
            _ => match eat_token(tokens, None, word_lut) {
                Ok(nar) => {
                    res.push(nar.node);
                    tokens = nar.tokens;
                }
                Err(e) => return Err(e),
            },
        }
    }
}

fn eat_vector<'a>(
    t: &'a [Token<'a>],
    meta: Option<NodeMeta>,
    word_lut: &mut WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    loop {
        match tokens[0] {
            Token::SquareBracketEnd => {
                return Ok(NodeAndRemainder {
                    node: Node::Vector(res, meta),
                    tokens: &tokens[1..],
                });
            }
            _ => match eat_token(tokens, None, word_lut) {
                Ok(nar) => {
                    res.push(nar.node);
                    tokens = nar.tokens;
                }
                Err(e) => return Err(e),
            },
        }
    }
}

fn eat_alterable<'a>(t: &'a [Token<'a>], word_lut: &mut WordLut) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;

    // possible parameter_prefix
    let mut parameter_prefix: Vec<Node> = Vec::new();
    loop {
        match tokens[0] {
            Token::Whitespace(ws) => {
                parameter_prefix.push(Node::Whitespace(ws.to_string(), None));
                tokens = &tokens[1..];
            }
            Token::Comment(comment) => {
                parameter_prefix.push(Node::Comment(comment.to_string(), None));
                tokens = &tokens[1..];
            }
            _ => break,
        }
    }

    let default_expression = eat_token(tokens, None, word_lut)?;
    tokens = default_expression.tokens;

    // parameter ast (incl. whitespace, comments etc)
    let mut parameter_ast: Vec<Node> = Vec::new();
    loop {
        match tokens[0] {
            Token::CurlyBracketEnd => {
                // construct the NodeMeta
                let meta = Some(NodeMeta {
                    gene: None,
                    parameter_ast,
                    parameter_prefix,
                });

                // add the correct meta information to the parsed default node
                let default_with_meta = match default_expression.node {
                    Node::List(ns, _) => Node::List(ns, meta),
                    Node::Vector(ns, _) => Node::Vector(ns, meta),
                    Node::Float(f, s, _) => Node::Float(f, s, meta),
                    Node::Name(s, i, _) => Node::Name(s, i, meta),
                    Node::Label(s, i, _) => Node::Label(s, i, meta),
                    Node::String(s, _) => Node::String(s, meta),
                    Node::Whitespace(s, _) => Node::Whitespace(s, meta),
                    Node::Comment(s, _) => Node::Comment(s, meta),
                };

                return Ok(NodeAndRemainder {
                    node: default_with_meta,
                    tokens: &tokens[1..],
                });
            }
            _ => match eat_token(tokens, None, word_lut) {
                Ok(nar) => {
                    parameter_ast.push(nar.node);
                    tokens = nar.tokens;
                }
                Err(e) => return Err(e),
            },
        }
    }

    // return Err(Error::GeneralError)
}

fn eat_quoted_form<'a>(
    t: &'a [Token<'a>],
    meta: Option<NodeMeta>,
    word_lut: &mut WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    let q = "quote".to_string();
    let qi = word_lut.get_or_add(&q);
    res.push(Node::Name(q, qi, None));
    res.push(Node::Whitespace(" ".to_string(), None));

    match eat_token(tokens, None, word_lut) {
        Ok(nar) => {
            res.push(nar.node);
            tokens = nar.tokens;
        }
        Err(e) => return Err(e),
    }

    Ok(NodeAndRemainder {
        node: Node::List(res, meta),
        tokens: &tokens[..],
    })
}

fn eat_token<'a>(
    tokens: &'a [Token<'a>],
    meta: Option<NodeMeta>,
    word_lut: &mut WordLut,
) -> Result<NodeAndRemainder<'a>> {
    match tokens[0] {
        Token::Name(txt) => {
            let t = txt.to_string();
            let ti = word_lut.get_or_add(&t);
            if tokens.len() > 1 && tokens[1] == Token::Colon {
                Ok(NodeAndRemainder {
                    node: Node::Label(t, ti, meta),
                    tokens: &tokens[2..],
                })
            } else {
                Ok(NodeAndRemainder {
                    node: Node::Name(t, ti, meta),
                    tokens: &tokens[1..],
                })
            }
        }
        Token::Number(txt) => match txt.parse::<f32>() {
            Ok(f) => Ok(NodeAndRemainder {
                node: Node::Float(f, txt.to_string(), meta),
                tokens: &tokens[1..],
            }),
            Err(_) => Err(Error::ParserUnableToParseFloat(txt.to_string())),
        },
        Token::Whitespace(ws) => Ok(NodeAndRemainder {
            node: Node::Whitespace(ws.to_string(), None),
            tokens: &tokens[1..],
        }),
        Token::Comment(comment) => Ok(NodeAndRemainder {
            node: Node::Comment(comment.to_string(), None),
            tokens: &tokens[1..],
        }),
        Token::Quote => eat_quoted_form(&tokens[1..], meta, word_lut),
        Token::ParenStart => eat_list(&tokens[1..], meta, word_lut),
        Token::SquareBracketStart => eat_vector(&tokens[1..], meta, word_lut),
        Token::CurlyBracketStart => eat_alterable(&tokens[1..], word_lut),
        _ => Err(Error::ParserHandledToken(format!("{:?}", tokens[0]))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ast(s: &str) -> Vec<Node> {
        let (tree, _word_lut) = parse(s).unwrap();
        tree
    }

    #[test]
    fn bug_alterable_square_brackets() {
        let s = "
(define
  coords1 {[[1 2]
            [3 4]]
           (gen/2d min: -500 max: 500)})
";
        match parse(s) {
            Ok((_ast, _word_lut)) => assert_eq!(true, true),
            Err(_e) => assert_eq!(false, false),
        };
    }

    #[test]
    fn test_parser() {
        assert_eq!(
            ast("hello"),
            [Node::Name("hello".to_string(), Name::new(0), None)]
        );
        assert_eq!(
            ast("hello world"),
            [
                Node::Name("hello".to_string(), Name::new(0), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), Name::new(1), None)
            ]
        );
        assert_eq!(
            ast("hello: world"),
            [
                Node::Label("hello".to_string(), Name::new(0), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), Name::new(1), None)
            ]
        );
        assert_eq!(
            ast("42 102"),
            [
                Node::Float(42.0, "42".to_string(), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Float(102.0, "102".to_string(), None)
            ]
        );

        assert_eq!(
            ast("hello world ; some comment"),
            [
                Node::Name("hello".to_string(), Name::new(0), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), Name::new(1), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Comment(" some comment".to_string(), None)
            ]
        );

        assert_eq!(
            ast("(hello world)"),
            [Node::List(
                vec![
                    Node::Name("hello".to_string(), Name::new(0), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), Name::new(1), None)
                ],
                None
            )]
        );

        assert_eq!(
            ast("'3"),
            [Node::List(
                vec![
                    Node::Name("quote".to_string(), Name::new(153), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Float(3.0, "3".to_string(), None)
                ],
                None
            )]
        );

        assert_eq!(
            ast("(hello world (1 2 3))"),
            [Node::List(
                vec![
                    Node::Name("hello".to_string(), Name::new(0), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), Name::new(1), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::List(
                        vec![
                            Node::Float(1.0, "1".to_string(), None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(2.0, "2".to_string(), None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(3.0, "3".to_string(), None)
                        ],
                        None
                    )
                ],
                None
            )]
        );

        assert_eq!(
            ast("(hello world [1 2 3])"),
            [Node::List(
                vec![
                    Node::Name("hello".to_string(), Name::new(0), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), Name::new(1), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Vector(
                        vec![
                            Node::Float(1.0, "1".to_string(), None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(2.0, "2".to_string(), None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(3.0, "3".to_string(), None)
                        ],
                        None
                    )
                ],
                None
            )]
        );

        assert_eq!(
            ast("hello { 5 (gen/scalar)}"),
            [
                Node::Name("hello".to_string(), Name::new(0), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Float(
                    5.0,
                    "5".to_string(),
                    Some(NodeMeta {
                        gene: None,
                        parameter_ast: vec![
                            Node::Whitespace(" ".to_string(), None),
                            Node::List(
                                vec![Node::Name(
                                    "gen/scalar".to_string(),
                                    Name::from(Native::GenScalar),
                                    None
                                )],
                                None
                            )
                        ],
                        parameter_prefix: vec![Node::Whitespace(" ".to_string(), None)]
                    })
                )
            ]
        );
    }

    #[test]
    fn test_parser_native() {
        assert_eq!(
            ast("(rect width: 300)"),
            [Node::List(
                vec![
                    Node::Name("rect".to_string(), Name::from(Native::Rect), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Label("width".to_string(), Name::from(Keyword::Width), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Float(300.0, "300".to_string(), None),
                ],
                None
            )]
        );
    }
}
