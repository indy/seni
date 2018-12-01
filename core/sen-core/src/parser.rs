// Copyright (C) 2018 Inderjit Gill

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

use error::{SenResult, SenError};
use lexer::{Token, tokenize};

struct Gene {
    this_is_a_placeholder: i32,
}

struct NodeMeta {
    gene: Option<Gene>,
    parameter_ast: Vec<Node>,
    parameter_prefix: Vec<Node>, // todo: couldn't this just be a String?
}

#[derive(Debug, PartialEq)]
pub enum Node {
    List(Vec<Node>),
    Vector(Vec<Node>),
    Float(f32),
    Name(String),
    Label(String),
    String(String),
    Whitespace(String),
    Comment(String),
}

struct NodeAndRemainder<'a> {
    node: Node,
    tokens: &'a [Token<'a>],
}

// At the first token after a Token::ParenStart
//
fn eat_list<'a>(t: &'a [Token<'a>]) -> SenResult<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    loop {
        match tokens[0] {
            Token::ParenEnd => return Ok(NodeAndRemainder {
                node: Node::List(res),
                tokens: &tokens[1..],
            }),
            _ => {
                match eat_token(tokens) {
                    Ok(nar) => {
                        res.push(nar.node);
                        tokens = nar.tokens;
                    },
                    Err(e) => return Err(e)
                }
            }
        }
    }
}

fn eat_vector<'a>(t: &'a [Token<'a>]) -> SenResult<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    loop {
        match tokens[0] {
            Token::SquareBracketEnd => return Ok(NodeAndRemainder {
                node: Node::Vector(res),
                tokens: &tokens[1..],
            }),
            _ => {
                match eat_token(tokens) {
                    Ok(nar) => {
                        res.push(nar.node);
                        tokens = nar.tokens;
                    },
                    Err(e) => return Err(e)
                }
            }
        }
    }
}

fn eat_token<'a>(tokens: &'a[Token<'a>]) -> SenResult<NodeAndRemainder<'a>> {
    match tokens[0] {
        Token::Name(txt) =>
            if tokens.len() > 1 && tokens[1] == Token::Colon {
                Ok(NodeAndRemainder {
                    node: Node::Label(txt.to_string()),
                    tokens: &tokens[2..],
                })
            } else {
                Ok(NodeAndRemainder {
                    node: Node::Name(txt.to_string()),
                    tokens: &tokens[1..],
                })
            },
        Token::Number(txt) => {
            match txt.parse::<f32>() {
                Ok(f) => Ok(NodeAndRemainder {
                    node: Node::Float(f),
                    tokens: &tokens[1..],
                }),
                Err(_) => Err(SenError::ParserUnableToParseFloat(txt.to_string()))
            }
        },
        Token::Whitespace(ws) => Ok(NodeAndRemainder {
            node: Node::Whitespace(ws.to_string()),
            tokens: &tokens[1..],
        }),
        Token::Comment(comment) => Ok(NodeAndRemainder {
            node: Node::Comment(comment.to_string()),
            tokens: &tokens[1..],
        }),
        Token::ParenStart => eat_list(&tokens[1..]),
        Token::SquareBracketStart => eat_vector(&tokens[1..]),
        _ => Err(SenError::ParserHandledToken),
    }
}

pub fn parse(s: &str) -> SenResult<Vec<Node>> {
    let t = tokenize(s)?;

    let mut tokens = t.as_slice();
    let mut res = Vec::new();

    while tokens.len() > 0 {
        match eat_token(tokens) {
            Ok(nar) => {
                res.push(nar.node);
                tokens = nar.tokens;
            },
            Err(e) => return Err(e)
        }
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ast(s: &str) -> Vec<Node> {
        parse(s).unwrap()
    }

    #[test]
    fn test_parser() {
        assert_eq!(ast("hello"),
                   [Node::Name("hello".to_string())]);
        assert_eq!(ast("hello world"),
                   [Node::Name("hello".to_string()),
                    Node::Whitespace(" ".to_string()),
                    Node::Name("world".to_string())]);
        assert_eq!(ast("hello: world"),
                   [Node::Label("hello".to_string()),
                    Node::Whitespace(" ".to_string()),
                    Node::Name("world".to_string())]);
        assert_eq!(ast("42 102"),
                   [Node::Float(42.0),
                    Node::Whitespace(" ".to_string()),
                    Node::Float(102.0)]);

        assert_eq!(ast("hello world ; some comment"),
                   [Node::Name("hello".to_string()),
                    Node::Whitespace(" ".to_string()),
                    Node::Name("world".to_string()),
                    Node::Whitespace(" ".to_string()),
                    Node::Comment(" some comment".to_string())]);

        assert_eq!(ast("(hello world)"),
                   [Node::List(vec![Node::Name("hello".to_string()),
                                    Node::Whitespace(" ".to_string()),
                                    Node::Name("world".to_string())])]);

        assert_eq!(ast("(hello world (1 2 3))"),
                   [Node::List(vec![Node::Name("hello".to_string()),
                                    Node::Whitespace(" ".to_string()),
                                    Node::Name("world".to_string()),
                                    Node::Whitespace(" ".to_string()),
                                    Node::List(vec![Node::Float(1.0),
                                                    Node::Whitespace(" ".to_string()),
                                                    Node::Float(2.0),
                                                    Node::Whitespace(" ".to_string()),
                                                    Node::Float(3.0)])])]);


        assert_eq!(ast("(hello world [1 2 3])"),
                   [Node::List(vec![Node::Name("hello".to_string()),
                                    Node::Whitespace(" ".to_string()),
                                    Node::Name("world".to_string()),
                                    Node::Whitespace(" ".to_string()),
                                    Node::Vector(vec![Node::Float(1.0),
                                                      Node::Whitespace(" ".to_string()),
                                                      Node::Float(2.0),
                                                      Node::Whitespace(" ".to_string()),
                                                      Node::Float(3.0)])])]);

    }
}
