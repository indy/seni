// Copyright (C) 2019 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::error::{Error, Result};
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::lexer::{tokenize, Token};
use crate::native::Native;
use crate::node::{Node, NodeGene, NodeLocation, NodeMeta};
use log::error;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct WordLut {
    // requires a builtin hashmap (function names reserved by the builtin api)
    // a keyword hashmap (keywords + constants + common arguments to builtin api functions)
    // a word hashmap (user defined names and labels)
    word_to_iname: HashMap<String, Iname>,
    word_count: i32,

    iname_to_word: HashMap<Iname, String>,
    iname_to_native: HashMap<Iname, String>,
    iname_to_keyword: HashMap<Iname, String>,
}

struct NodeAndRemainder<'a> {
    node: Node,
    loc: NodeLocation,
    tokens: &'a [Token<'a>],
}

#[derive(PartialEq)]
enum AlterableCheck {
    Yes,
    No,
}

fn is_name_or_keyword(s: &str) -> bool {
    if let Ok(_) = Native::from_str(s) {
        return true;
    }
    if let Ok(_) = Keyword::from_str(s) {
        return true;
    }
    false
}

impl Default for WordLut {
    fn default() -> WordLut {
        // native
        let mut n: HashMap<Iname, String> = HashMap::new();
        for nat in Native::iter() {
            n.insert(Iname::from(nat), nat.to_string());
        }

        // keyword
        let mut k: HashMap<Iname, String> = HashMap::new();
        for kw in Keyword::iter() {
            k.insert(Iname::from(kw), kw.to_string());
        }

        WordLut {
            word_to_iname: HashMap::new(),
            word_count: 0,

            iname_to_word: HashMap::new(),
            iname_to_native: n,
            iname_to_keyword: k,
        }
    }
}

impl WordLut {
    pub fn new(tokens: &[Token]) -> Self {
        let mut word_lut: WordLut = Default::default();
        let mut words: HashSet<String> = HashSet::new();

        for t in tokens {
            match t {
                Token::Name(txt) if !is_name_or_keyword(&txt) => {
                    words.insert(txt.to_string());
                }
                Token::String(txt) if !is_name_or_keyword(&txt) => {
                    words.insert(txt.to_string());
                }
                _ => {}
            }
        }

        // sort the set of words into alphabetical order before assigning Inames
        // this ensures that they get the same Iname regardless of their position
        // in the script (which could change depending on the genotype used)
        //

        let mut word_list: Vec<&String> = words.iter().collect();
        word_list.sort();

        for (i, word) in word_list.iter().enumerate() {
            let iname = Iname::new(i as i32);
            word_lut.word_to_iname.insert(word.to_string(), iname);
            word_lut.iname_to_word.insert(iname, word.to_string());
        }

        word_lut
    }

    pub fn get_string_from_name(&self, name: Iname) -> Option<&String> {
        if let Some(s) = self.iname_to_keyword.get(&name) {
            // 1st check the keywords
            Some(s)
        } else if let Some(s) = self.iname_to_native.get(&name) {
            // 2nd check the native api
            Some(s)
        } else {
            // finally check the iname_to_word
            self.iname_to_word.get(&name)
        }
    }

    fn get(&self, s: &str) -> Result<Iname> {
        if let Some(i) = self.get_name_from_string(s) {
            return Ok(i);
        }

        Err(Error::Parser)
    }

    fn get_name_from_string(&self, s: &str) -> Option<Iname> {
        // 1st check the keywords
        if let Ok(kw) = Keyword::from_str(s) {
            return Some(Iname::from(kw));
        }

        // 2nd check the native api
        if let Ok(n) = Native::from_str(s) {
            return Some(Iname::from(n));
        }

        // finally check word_to_iname
        if let Some(i) = self.word_to_iname.get(s) {
            return Some(*i);
        }

        None
    }

    // used to populate Program.Data.strings
    pub fn get_script_inames(&self) -> BTreeMap<Iname, String> {
        let mut res: BTreeMap<Iname, String> = BTreeMap::new();

        for (k, v) in &self.iname_to_word {
            res.insert(*k, v.clone());
        }

        res
    }
}

pub fn parse(s: &str) -> Result<(Vec<Node>, WordLut)> {
    let t = tokenize(s)?;

    let mut tokens = t.as_slice();
    let mut res = Vec::new();

    let mut loc = NodeLocation {
        line: 1,
        character: 1,
    };

    let word_lut = WordLut::new(tokens);

    while !tokens.is_empty() {
        match eat_token(tokens, loc, None, &word_lut, AlterableCheck::Yes) {
            Ok(nar) => {
                res.push(nar.node);
                tokens = nar.tokens;
                loc = nar.loc;
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
    loc: NodeLocation,
    gene_info: Option<NodeGene>,
    word_lut: &WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    let mut loc2 = NodeLocation {
        line: loc.line,
        character: loc.character + 1,
    };

    loop {
        match tokens[0] {
            Token::ParenEnd => {
                loc2.character += 1;
                return Ok(NodeAndRemainder {
                    node: Node::List(NodeMeta { loc, gene_info }, res),
                    loc: loc2,
                    tokens: &tokens[1..],
                });
            }
            _ => match eat_token(tokens, loc2, None, word_lut, AlterableCheck::Yes) {
                Ok(nar) => {
                    res.push(nar.node);
                    loc2 = nar.loc;
                    tokens = nar.tokens;
                }
                Err(e) => return Err(e),
            },
        }
    }
}

fn eat_vector<'a>(
    t: &'a [Token<'a>],
    loc: NodeLocation,
    gene_info: Option<NodeGene>,
    word_lut: &WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    let mut loc2 = NodeLocation {
        line: loc.line,
        character: loc.character + 1,
    };

    loop {
        match tokens[0] {
            Token::SquareBracketEnd => {
                loc2.character += 1;
                return Ok(NodeAndRemainder {
                    node: Node::Vector(NodeMeta { loc, gene_info }, res),
                    loc: loc2,
                    tokens: &tokens[1..],
                });
            }
            _ => match eat_token(tokens, loc2, None, word_lut, AlterableCheck::Yes) {
                Ok(nar) => {
                    res.push(nar.node);
                    loc2 = nar.loc;
                    tokens = nar.tokens;
                }
                Err(e) => return Err(e),
            },
        }
    }
}

fn eat_quoted_form<'a>(
    t: &'a [Token<'a>],
    loc: NodeLocation,
    gene_info: Option<NodeGene>,
    word_lut: &WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    let q = "quote".to_string();
    let qi = word_lut.get(&q)?;
    res.push(Node::Name(
        NodeMeta {
            loc,
            gene_info: None,
        },
        q,
        qi,
    ));
    res.push(Node::Whitespace(
        NodeMeta {
            loc,
            gene_info: None,
        },
        " ".to_string(),
    ));

    let mut loc2 = NodeLocation {
        line: loc.line,
        character: loc.character + 1,
    };

    match eat_token(tokens, loc2, None, word_lut, AlterableCheck::Yes) {
        Ok(nar) => {
            res.push(nar.node);
            tokens = nar.tokens;
            loc2 = nar.loc;
        }
        Err(e) => return Err(e),
    }

    Ok(NodeAndRemainder {
        node: Node::List(NodeMeta { loc, gene_info }, res),
        loc: loc2,
        tokens: &tokens[..],
    })
}

fn eat_token<'a>(
    tokens: &'a [Token<'a>],
    loc: NodeLocation,
    gene_info: Option<NodeGene>,
    word_lut: &WordLut,
    check_for_alterable: AlterableCheck,
) -> Result<NodeAndRemainder<'a>> {
    let nar = match tokens[0] {
        Token::Name(txt) => {
            let t = txt.to_string();
            let ti = word_lut.get(&t)?;
            if tokens.len() > 1 && tokens[1] == Token::Colon {
                NodeAndRemainder {
                    node: Node::Label(NodeMeta { loc, gene_info }, t, ti),
                    loc: NodeLocation {
                        line: loc.line,
                        character: loc.character + txt.len() + 1,
                    },
                    tokens: &tokens[2..],
                }
            } else if tokens.len() > 1 && tokens[1] == Token::Dot {
                NodeAndRemainder {
                    node: Node::FromName(NodeMeta { loc, gene_info }, t, ti),
                    loc: NodeLocation {
                        line: loc.line,
                        character: loc.character + txt.len() + 1,
                    },
                    tokens: &tokens[2..],
                }
            } else {
                NodeAndRemainder {
                    node: Node::Name(NodeMeta { loc, gene_info }, t, ti),
                    loc: NodeLocation {
                        line: loc.line,
                        character: loc.character + txt.len(),
                    },
                    tokens: &tokens[1..],
                }
            }
        }
        Token::String(txt) => {
            let t = txt.to_string();
            let ti = word_lut.get(&t)?;
            NodeAndRemainder {
                node: Node::String(NodeMeta { loc, gene_info }, t, ti),
                loc: NodeLocation {
                    line: loc.line,
                    character: loc.character + txt.len(),
                },
                tokens: &tokens[1..],
            }
        }
        Token::Number(txt) => match txt.parse::<f32>() {
            Ok(f) => NodeAndRemainder {
                node: Node::Float(NodeMeta { loc, gene_info }, f, txt.to_string()),
                loc: NodeLocation {
                    line: loc.line,
                    character: loc.character + txt.len(),
                },
                tokens: &tokens[1..],
            },
            Err(_) => {
                error!("Parser unable to parse float: {}", txt);
                return Err(Error::Parser);
            }
        },
        Token::Tilde => NodeAndRemainder {
            node: Node::Tilde(NodeMeta {
                loc,
                gene_info: None,
            }),
            loc: NodeLocation {
                line: loc.line,
                character: loc.character + 1,
            },
            tokens: &tokens[1..],
        },
        Token::Newline => NodeAndRemainder {
            node: Node::Whitespace(
                NodeMeta {
                    loc,
                    gene_info: None,
                },
                "\n".to_string(),
            ),
            loc: NodeLocation {
                line: loc.line + 1,
                character: 1,
            },
            tokens: &tokens[1..],
        },
        Token::Whitespace(ws) => NodeAndRemainder {
            node: Node::Whitespace(
                NodeMeta {
                    loc,
                    gene_info: None,
                },
                ws.to_string(),
            ),
            loc: NodeLocation {
                line: loc.line,
                character: loc.character + ws.len(),
            },
            tokens: &tokens[1..],
        },
        Token::Comment(comment) => NodeAndRemainder {
            node: Node::Comment(
                NodeMeta {
                    loc,
                    gene_info: None,
                },
                comment.to_string(),
            ),
            loc: NodeLocation {
                line: loc.line,
                character: loc.character + comment.len(),
            },
            tokens: &tokens[1..],
        },
        Token::Quote => eat_quoted_form(&tokens[1..], loc, gene_info, word_lut)?,
        Token::ParenStart => eat_list(&tokens[1..], loc, gene_info, word_lut)?,
        Token::SquareBracketStart => eat_vector(&tokens[1..], loc, gene_info, word_lut)?,
        _ => {
            error!("ParserHandledToken {:?}", tokens[0]);
            return Err(Error::Parser);
        }
    };

    if check_for_alterable == AlterableCheck::Yes {
        let mut iter = nar
            .tokens
            .iter()
            .skip_while(|&x| x.is_whitespace() || x.is_comment() || x.is_newline());
        if let Some(i) = iter.next() {
            if i.is_tilde() {
                // the current node is alterable
                return augment_node_with_alterable(nar, word_lut);
            }
        }
    }

    // get here if there isn't a tilde following the current token
    Ok(nar)
}

fn augment_node_with_alterable<'a>(
    nar: NodeAndRemainder<'a>,
    word_lut: &WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut parameter_prefix: Vec<Node> = vec![];
    let mut parameter_ast: Vec<Node> = vec![];
    let mut loc = nar.loc;
    let mut tokens = nar.tokens;

    loop {
        match eat_token(tokens, loc, None, word_lut, AlterableCheck::No) {
            Ok(nar) => {
                tokens = nar.tokens;
                loc = nar.loc;
                if nar.node.is_whitespace() || nar.node.is_comment() || nar.node.is_tilde() {
                    parameter_prefix.push(nar.node);
                } else {
                    // this is the first non whitespace, comment or tilde token.
                    // i.e. this is the parameter_ast
                    parameter_ast.push(nar.node);
                    break;
                }
            }
            Err(e) => return Err(e),
        }
    }

    let meta = NodeMeta {
        loc: nar.node.get_location(),
        gene_info: Some(NodeGene {
            gene: None,
            parameter_ast,
            parameter_prefix,
        }),
    };

    let node_with_meta = match nar.node {
        Node::List(_, ns) => Node::List(meta, ns),
        Node::Vector(_, ns) => Node::Vector(meta, ns),
        Node::Float(_, f, s) => Node::Float(meta, f, s),
        Node::FromName(_, s, i) => Node::FromName(meta, s, i),
        Node::Name(_, s, i) => Node::Name(meta, s, i),
        Node::Label(_, s, i) => Node::Label(meta, s, i),
        Node::String(_, s, i) => Node::String(meta, s, i),
        Node::Tilde(_) => Node::Tilde(meta),
        Node::Whitespace(_, s) => Node::Whitespace(meta, s),
        Node::Comment(_, s) => Node::Comment(meta, s),
    };

    Ok(NodeAndRemainder {
        node: node_with_meta,
        loc,
        tokens,
    })
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

    fn meta_loc(line: usize, character: usize) -> NodeMeta {
        NodeMeta {
            loc: NodeLocation { line, character },
            gene_info: None,
        }
    }

    #[test]
    fn test_parser_names() {
        assert_eq!(
            ast("hello"),
            [Node::Name(
                meta_loc(1, 1),
                "hello".to_string(),
                Iname::new(0)
            )]
        );
        assert_eq!(
            ast("hello world"),
            [
                Node::Name(meta_loc(1, 1), "hello".to_string(), Iname::new(0)),
                Node::Whitespace(meta_loc(1, 6), " ".to_string()),
                Node::Name(meta_loc(1, 7), "world".to_string(), Iname::new(1))
            ]
        );
        assert_eq!(
            ast("hello\nworld"),
            [
                Node::Name(meta_loc(1, 1), "hello".to_string(), Iname::new(0)),
                Node::Whitespace(meta_loc(1, 6), "\n".to_string()),
                Node::Name(meta_loc(2, 1), "world".to_string(), Iname::new(1))
            ]
        );
    }

    #[test]
    fn test_parser_label() {
        assert_eq!(
            ast("hello: world"),
            [
                Node::Label(meta_loc(1, 1), "hello".to_string(), Iname::new(0)),
                Node::Whitespace(meta_loc(1, 7), " ".to_string()),
                Node::Name(meta_loc(1, 8), "world".to_string(), Iname::new(1))
            ]
        );
    }

    #[test]
    fn test_parser_numbers() {
        assert_eq!(
            ast("42 102"),
            [
                Node::Float(meta_loc(1, 1), 42.0, "42".to_string()),
                Node::Whitespace(meta_loc(1, 3), " ".to_string()),
                Node::Float(meta_loc(1, 4), 102.0, "102".to_string())
            ]
        );
    }

    #[test]
    fn test_parser_comment() {
        assert_eq!(
            ast("hello world ; some comment"),
            [
                Node::Name(meta_loc(1, 1), "hello".to_string(), Iname::new(0)),
                Node::Whitespace(meta_loc(1, 6), " ".to_string()),
                Node::Name(meta_loc(1, 7), "world".to_string(), Iname::new(1)),
                Node::Whitespace(meta_loc(1, 12), " ".to_string()),
                Node::Comment(meta_loc(1, 13), " some comment".to_string())
            ]
        );
    }

    #[test]
    fn test_parser_list() {
        assert_eq!(
            ast("(hello world)"),
            [Node::List(
                meta_loc(1, 1),
                vec![
                    Node::Name(meta_loc(1, 2), "hello".to_string(), Iname::new(0)),
                    Node::Whitespace(meta_loc(1, 7), " ".to_string()),
                    Node::Name(meta_loc(1, 8), "world".to_string(), Iname::new(1))
                ],
            )]
        );

        assert_eq!(
            ast("(hello world) ; another comment"),
            [
                Node::List(
                    meta_loc(1, 1),
                    vec![
                        Node::Name(meta_loc(1, 2), "hello".to_string(), Iname::new(0)),
                        Node::Whitespace(meta_loc(1, 7), " ".to_string()),
                        Node::Name(meta_loc(1, 8), "world".to_string(), Iname::new(1)),
                    ],
                ),
                Node::Whitespace(meta_loc(1, 14), " ".to_string()),
                Node::Comment(meta_loc(1, 15), " another comment".to_string())
            ]
        );

        assert_eq!(
            ast("(bitmap \"foo.png\")"),
            [Node::List(
                meta_loc(1, 1),
                vec![
                    Node::Name(meta_loc(1, 2), "bitmap".to_string(), Iname::new(0)),
                    Node::Whitespace(meta_loc(1, 8), " ".to_string()),
                    Node::String(meta_loc(1, 9), "foo.png".to_string(), Iname::new(1))
                ],
            )]
        );

        assert_eq!(
            ast("(hello world (1 2 3))"),
            [Node::List(
                meta_loc(1, 1),
                vec![
                    Node::Name(meta_loc(1, 2), "hello".to_string(), Iname::new(0)),
                    Node::Whitespace(meta_loc(1, 7), " ".to_string()),
                    Node::Name(meta_loc(1, 8), "world".to_string(), Iname::new(1)),
                    Node::Whitespace(meta_loc(1, 13), " ".to_string()),
                    Node::List(
                        meta_loc(1, 14),
                        vec![
                            Node::Float(meta_loc(1, 15), 1.0, "1".to_string()),
                            Node::Whitespace(meta_loc(1, 16), " ".to_string()),
                            Node::Float(meta_loc(1, 17), 2.0, "2".to_string()),
                            Node::Whitespace(meta_loc(1, 18), " ".to_string()),
                            Node::Float(meta_loc(1, 19), 3.0, "3".to_string())
                        ],
                    )
                ],
            )]
        );

        assert_eq!(
            ast("(hello world [1 2 3])"),
            [Node::List(
                meta_loc(1, 1),
                vec![
                    Node::Name(meta_loc(1, 2), "hello".to_string(), Iname::new(0)),
                    Node::Whitespace(meta_loc(1, 7), " ".to_string()),
                    Node::Name(meta_loc(1, 8), "world".to_string(), Iname::new(1)),
                    Node::Whitespace(meta_loc(1, 13), " ".to_string()),
                    Node::Vector(
                        meta_loc(1, 14),
                        vec![
                            Node::Float(meta_loc(1, 15), 1.0, "1".to_string()),
                            Node::Whitespace(meta_loc(1, 16), " ".to_string()),
                            Node::Float(meta_loc(1, 17), 2.0, "2".to_string()),
                            Node::Whitespace(meta_loc(1, 18), " ".to_string()),
                            Node::Float(meta_loc(1, 19), 3.0, "3".to_string())
                        ],
                    )
                ],
            )]
        );
    }

    #[test]
    fn test_parser_quote() {
        assert_eq!(
            ast("'3"),
            [Node::List(
                meta_loc(1, 1),
                vec![
                    Node::Name(meta_loc(1, 1), "quote".to_string(), Iname::new(153)),
                    Node::Whitespace(meta_loc(1, 1), " ".to_string()),
                    Node::Float(meta_loc(1, 2), 3.0, "3".to_string())
                ],
            )]
        );
    }

    #[test]
    fn test_parser_alterable_tilde() {
        assert_eq!(
            ast("hello 5 ~ (gen/scalar) foo"),
            [
                Node::Name(meta_loc(1, 1), "hello".to_string(), Iname::new(1)),
                Node::Whitespace(meta_loc(1, 6), " ".to_string()),
                Node::Float(
                    NodeMeta {
                        loc: NodeLocation {
                            line: 1,
                            character: 7
                        },
                        gene_info: Some(NodeGene {
                            gene: None,
                            parameter_ast: vec![Node::List(
                                meta_loc(1, 11),
                                vec![Node::Name(
                                    meta_loc(1, 12),
                                    "gen/scalar".to_string(),
                                    Iname::from(Native::GenScalar),
                                )],
                            )],
                            parameter_prefix: vec![
                                Node::Whitespace(meta_loc(1, 8), " ".to_string()),
                                Node::Tilde(meta_loc(1, 9)),
                                Node::Whitespace(meta_loc(1, 10), " ".to_string())
                            ]
                        })
                    },
                    5.0,
                    "5".to_string(),
                ),
                Node::Whitespace(meta_loc(1, 23), " ".to_string()),
                Node::Name(meta_loc(1, 24), "foo".to_string(), Iname::new(0)),
            ]
        );
    }

    // todo: also test: can have a vector of vectors, top level should have a gen/2d
    #[test]
    fn test_parser_alterable_vector_tilde() {
        assert_eq!(
            ast("hello [7 8] ~ (gen/scalar)"),
            [
                Node::Name(meta_loc(1, 1), "hello".to_string(), Iname::new(0)),
                Node::Whitespace(meta_loc(1, 6), " ".to_string()),
                Node::Vector(
                    NodeMeta {
                        loc: NodeLocation {
                            line: 1,
                            character: 7
                        },
                        gene_info: Some(NodeGene {
                            gene: None,
                            parameter_ast: vec![Node::List(
                                meta_loc(1, 15),
                                vec![Node::Name(
                                    meta_loc(1, 16),
                                    "gen/scalar".to_string(),
                                    Iname::from(Native::GenScalar),
                                )],
                            )],
                            parameter_prefix: vec![
                                Node::Whitespace(meta_loc(1, 12), " ".to_string()),
                                Node::Tilde(meta_loc(1, 13)),
                                Node::Whitespace(meta_loc(1, 14), " ".to_string())
                            ]
                        })
                    },
                    vec![
                        Node::Float(meta_loc(1, 8), 7.0, "7".to_string()),
                        Node::Whitespace(meta_loc(1, 9), " ".to_string()),
                        Node::Float(meta_loc(1, 10), 8.0, "8".to_string())
                    ],
                ),
            ]
        );
    }

    #[test]
    fn test_parser_native() {
        assert_eq!(
            ast("(rect width: 300)"),
            [Node::List(
                meta_loc(1, 1),
                vec![
                    Node::Name(
                        meta_loc(1, 2),
                        "rect".to_string(),
                        Iname::from(Native::Rect)
                    ),
                    Node::Whitespace(meta_loc(1, 6), " ".to_string()),
                    Node::Label(
                        meta_loc(1, 7),
                        "width".to_string(),
                        Iname::from(Keyword::Width)
                    ),
                    Node::Whitespace(meta_loc(1, 13), " ".to_string()),
                    Node::Float(meta_loc(1, 14), 300.0, "300".to_string()),
                ],
            )]
        );
    }

    #[test]
    fn test_parser_from_name() {
        assert_eq!(
            ast("(some-vector.vector/length)"),
            [Node::List(
                meta_loc(1, 1),
                vec![
                    Node::FromName(meta_loc(1, 2), "some-vector".to_string(), Iname::new(0)),
                    Node::Name(
                        meta_loc(1, 14),
                        "vector/length".to_string(),
                        Iname::from(Native::VectorLength),
                    ),
                ],
            )]
        );
    }
}
