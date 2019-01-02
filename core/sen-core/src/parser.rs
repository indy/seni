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

use crate::error::{Error, Result};
use crate::keywords::string_to_keyword;
use crate::lexer::{tokenize, Token};
use crate::native::string_to_native;

#[derive(Debug, PartialEq)]
pub struct Gene {
    this_is_a_placeholder: i32,
}

#[derive(Debug, PartialEq)]
pub struct NodeMeta {
    gene: Option<Gene>,
    parameter_ast: Vec<Node>,
    parameter_prefix: Vec<Node>, // todo: couldn't this just be a String?
}

#[derive(Debug, PartialEq)]
pub enum Node {
    List(Vec<Node>, Option<NodeMeta>),
    Vector(Vec<Node>, Option<NodeMeta>),
    Float(f32, Option<NodeMeta>),
    Name(String, i32, Option<NodeMeta>),  // text, iname, meta
    Label(String, i32, Option<NodeMeta>), // text, iname, meta
    String(String, Option<NodeMeta>),
    Whitespace(String, Option<NodeMeta>),
    Comment(String, Option<NodeMeta>),
}

struct NodeAndRemainder<'a> {
    node: Node,
    tokens: &'a [Token<'a>],
}

pub struct WordLut {
    // requires a native hashmap (function names reserved by the native api)
    // a keyword hashmap (keywords + constants + common arguments to native api functions)
    // a word hashmap (user defined names and labels)
    word_ref: HashMap<String, i32>,
    word_count: i32,
}

impl WordLut {
    pub fn new() -> WordLut {
        WordLut {
            word_ref: HashMap::new(),
            word_count: 0,
        }
    }

    pub fn get(&mut self, s: &str) -> Option<i32> {
        // first check the native api
        if let Some(n) = string_to_native(s) {
            return Some(n as i32);
        }

        // 2nd check the keywords
        if let Some(kw) = string_to_keyword(s) {
            return Some(kw as i32);
        }

        // finally check/add to word_ref
        if let Some(i) = self.word_ref.get(s) {
            return Some(*i);
        }

        None
    }

    pub fn get_or_add(&mut self, s: &str) -> i32 {
        if let Some(i) = self.get(s) {
            return i;
        }

        self.word_ref.insert(s.to_string(), self.word_count);
        self.word_count += 1;
        self.word_count - 1
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
                })
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
                })
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

    // the main node
    let main_token = &tokens[..1];

    // skip the main node
    tokens = &tokens[1..];

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

                let res = eat_token(main_token, meta, word_lut)?;

                return Ok(NodeAndRemainder {
                    node: res.node,
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
                node: Node::Float(f, meta),
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
        _ => Err(Error::ParserHandledToken),
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
    fn test_parser() {
        assert_eq!(ast("hello"), [Node::Name("hello".to_string(), 0, None)]);
        assert_eq!(
            ast("hello world"),
            [
                Node::Name("hello".to_string(), 0, None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), 1, None)
            ]
        );
        assert_eq!(
            ast("hello: world"),
            [
                Node::Label("hello".to_string(), 0, None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), 1, None)
            ]
        );
        assert_eq!(
            ast("42 102"),
            [
                Node::Float(42.0, None),
                Node::Whitespace(" ".to_string(), None),
                Node::Float(102.0, None)
            ]
        );

        assert_eq!(
            ast("hello world ; some comment"),
            [
                Node::Name("hello".to_string(), 0, None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), 1, None),
                Node::Whitespace(" ".to_string(), None),
                Node::Comment(" some comment".to_string(), None)
            ]
        );

        assert_eq!(
            ast("(hello world)"),
            [Node::List(
                vec![
                    Node::Name("hello".to_string(), 0, None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), 1, None)
                ],
                None
            )]
        );

        assert_eq!(
            ast("'3"),
            [Node::List(
                vec![
                    Node::Name("quote".to_string(), 153, None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Float(3.0, None)
                ],
                None
            )]
        );

        assert_eq!(
            ast("(hello world (1 2 3))"),
            [Node::List(
                vec![
                    Node::Name("hello".to_string(), 0, None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), 1, None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::List(
                        vec![
                            Node::Float(1.0, None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(2.0, None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(3.0, None)
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
                    Node::Name("hello".to_string(), 0, None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), 1, None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Vector(
                        vec![
                            Node::Float(1.0, None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(2.0, None),
                            Node::Whitespace(" ".to_string(), None),
                            Node::Float(3.0, None)
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
                Node::Name("hello".to_string(), 0, None),
                Node::Whitespace(" ".to_string(), None),
                Node::Float(
                    5.0,
                    Some(NodeMeta {
                        gene: None,
                        parameter_ast: vec![
                            Node::Whitespace(" ".to_string(), None),
                            Node::List(vec![Node::Name("gen/scalar".to_string(), 385, None)], None)
                        ],
                        parameter_prefix: vec![Node::Whitespace(" ".to_string(), None)]
                    })
                )
            ]
        );
    }
}
