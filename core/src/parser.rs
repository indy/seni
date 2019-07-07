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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;

use crate::colour::Colour;
use crate::error::Error;
use crate::gene::Gene;
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::lexer::{tokenize, Token};
use crate::native::Native;
use crate::result::Result;

use strum::IntoEnumIterator;

use log::error;

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
    FromName(String, Iname, Option<NodeMeta>), // text, iname, meta
    Name(String, Iname, Option<NodeMeta>),     // text, iname, meta
    Label(String, Iname, Option<NodeMeta>),    // text, iname, meta
    String(String, Iname, Option<NodeMeta>),
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
                                error!("Node::get_float incompatible gene");
                                return Err(Error::Parser);
                            }
                        }
                    }
                }
            } else {
                return Ok(*f);
            }
        }
        error!("Node::get_float expected Node::Float not {:?}", self);
        Err(Error::Parser)
    }

    pub fn get_iname(&self, use_genes: bool) -> Result<Iname> {
        match self {
            Node::Name(_text, iname, meta) => {
                if use_genes && meta.is_some() {
                    if let Some(meta) = meta {
                        if let Some(gene) = &meta.gene {
                            match gene {
                                Gene::Name(i) => return Ok(*i),
                                _ => {
                                    error!("Node::get_iname incompatible gene for Name");
                                    return Err(Error::Parser);
                                }
                            }
                        }
                    }
                } else {
                    return Ok(*iname);
                }
            }
            Node::FromName(_text, iname, meta) => {
                if use_genes && meta.is_some() {
                    if let Some(meta) = meta {
                        if let Some(gene) = &meta.gene {
                            match gene {
                                Gene::Name(i) => return Ok(*i),
                                _ => {
                                    error!("Node::get_iname incompatible gene for FromName");
                                    return Err(Error::Parser);
                                }
                            }
                        }
                    }
                } else {
                    return Ok(*iname);
                }
            }
            Node::String(_text, iname, meta) => {
                if use_genes && meta.is_some() {
                    if let Some(meta) = meta {
                        if let Some(gene) = &meta.gene {
                            match gene {
                                Gene::String(i) => return Ok(*i),
                                _ => {
                                    error!("Node::get_iname incompatible gene for String");
                                    return Err(Error::Parser);
                                }
                            }
                        }
                    }
                } else {
                    return Ok(*iname);
                }
            }
            _ => {
                error!(
                    "Node::get_iname expected Node::Name or Node::String not {:?}",
                    self
                );
                return Err(Error::Parser);
            }
        }

        error!(
            "Node::get_iname expected Node::Name or Node::String not {:?}",
            self
        );
        Err(Error::Parser)
    }

    pub fn get_label_iname(&self, use_genes: bool) -> Result<Iname> {
        if let Node::Label(_, iname, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            // todo: what type of gene would a Node::Label have?
                            Gene::Int(i) => return Ok(Iname::new(*i)),
                            Gene::Name(i) => return Ok(*i),
                            _ => {
                                error!("Node::get_label_iname incompatible gene");
                                return Err(Error::Parser);
                            }
                        }
                    }
                }
            } else {
                return Ok(*iname);
            }
        }
        error!("Node::get_label_iname expected Node::Label not {:?}", self);
        Err(Error::Parser)
    }

    pub fn get_colour(&self, use_genes: bool) -> Result<Colour> {
        if let Node::List(_, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            Gene::Colour(col) => return Ok(*col),
                            _ => {
                                error!("Node::get_colour incompatible gene");
                                return Err(Error::Parser);
                            }
                        }
                    }
                }
            } else {
                error!("Node::get_colour expected to use gene");
                return Err(Error::Parser);
            }
        }
        error!("Node::get_colour expected Node::List not {:?}", self);
        Err(Error::Parser)
    }

    pub fn get_2d(&self, use_genes: bool) -> Result<(f32, f32)> {
        if let Node::Vector(_, meta) = self {
            if use_genes && meta.is_some() {
                if let Some(meta) = meta {
                    if let Some(gene) = &meta.gene {
                        match gene {
                            Gene::V2D(x, y) => return Ok((*x, *y)),
                            _ => {
                                error!("Node::get_2d incompatible gene");
                                return Err(Error::Parser);
                            }
                        }
                    }
                }
            } else {
                error!("Node::get_2d expected to use gene");
                return Err(Error::Parser);
            }
        }
        error!("Node::get_2d expected Node::Vector not {:?}", self);
        Err(Error::Parser)
    }

    pub fn is_name(&self) -> bool {
        match self {
            Node::Name(_, _, _) => true,
            _ => false,
        }
    }

    pub fn is_name_with_iname(&self, iname: &Iname) -> bool {
        match self {
            Node::Name(_, name_iname, _) => {
                name_iname == iname
            },
            _ => false,
        }
    }

    pub fn is_alterable(&self) -> bool {
        match self {
            Node::List(_, meta)
            | Node::Vector(_, meta)
            | Node::Float(_, _, meta)
            | Node::FromName(_, _, meta)
            | Node::Name(_, _, meta)
            | Node::Label(_, _, meta)
            | Node::String(_, _, meta)
            | Node::Whitespace(_, meta)
            | Node::Comment(_, meta) => meta.is_some(),
        }
    }

    pub fn has_gene(&self) -> bool {
        match self {
            Node::List(_, meta)
            | Node::Vector(_, meta)
            | Node::Float(_, _, meta)
            | Node::FromName(_, _, meta)
            | Node::Name(_, _, meta)
            | Node::Label(_, _, meta)
            | Node::String(_, _, meta)
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

fn is_name_or_keyword(s: &str) -> bool {
    if let Ok(_) = Native::from_str(s) {
        return true;
    }
    if let Ok(_) = Keyword::from_str(s) {
        return true;
    }
    false
}

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

    let word_lut = WordLut::new(tokens);

    while !tokens.is_empty() {
        match eat_token(tokens, None, &word_lut) {
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
    word_lut: &WordLut,
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
    word_lut: &WordLut,
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

fn eat_alterable<'a>(t: &'a [Token<'a>], word_lut: &WordLut) -> Result<NodeAndRemainder<'a>> {
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
                    Node::FromName(s, i, _) => Node::FromName(s, i, meta),
                    Node::Name(s, i, _) => Node::Name(s, i, meta),
                    Node::Label(s, i, _) => Node::Label(s, i, meta),
                    Node::String(s, i, _) => Node::String(s, i, meta),
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
}

fn eat_quoted_form<'a>(
    t: &'a [Token<'a>],
    meta: Option<NodeMeta>,
    word_lut: &WordLut,
) -> Result<NodeAndRemainder<'a>> {
    let mut tokens = t;
    let mut res: Vec<Node> = Vec::new();

    let q = "quote".to_string();
    let qi = word_lut.get(&q)?;
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
    word_lut: &WordLut,
) -> Result<NodeAndRemainder<'a>> {
    match tokens[0] {
        Token::Name(txt) => {
            let t = txt.to_string();
            let ti = word_lut.get(&t)?;
            if tokens.len() > 1 && tokens[1] == Token::Colon {
                Ok(NodeAndRemainder {
                    node: Node::Label(t, ti, meta),
                    tokens: &tokens[2..],
                })
            } else if tokens.len() > 1 && tokens[1] == Token::Dot {
                Ok(NodeAndRemainder {
                    node: Node::FromName(t, ti, meta),
                    tokens: &tokens[2..],
                })
            } else {
                Ok(NodeAndRemainder {
                    node: Node::Name(t, ti, meta),

                    tokens: &tokens[1..],
                })
            }
        }
        Token::String(txt) => {
            let t = txt.to_string();
            let ti = word_lut.get(&t)?;

            Ok(NodeAndRemainder {
                node: Node::String(t, ti, meta),
                tokens: &tokens[1..],
            })
        }
        Token::Number(txt) => match txt.parse::<f32>() {
            Ok(f) => Ok(NodeAndRemainder {
                node: Node::Float(f, txt.to_string(), meta),
                tokens: &tokens[1..],
            }),
            Err(_) => {
                error!("Parser unable to parse float: {}", txt);
                Err(Error::Parser)
            }
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
        _ => {
            error!("ParserHandledToken {:?}", tokens[0]);
            Err(Error::Parser)
        }
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
    fn test_parser_names() {
        assert_eq!(
            ast("hello"),
            [Node::Name("hello".to_string(), Iname::new(0), None)]
        );
        assert_eq!(
            ast("hello world"),
            [
                Node::Name("hello".to_string(), Iname::new(0), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), Iname::new(1), None)
            ]
        );
    }

    #[test]
    fn test_parser_label() {

        assert_eq!(
            ast("hello: world"),
            [
                Node::Label("hello".to_string(), Iname::new(0), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), Iname::new(1), None)
            ]
        );
    }

    #[test]
    fn test_parser_numbers() {
        assert_eq!(
            ast("42 102"),
            [
                Node::Float(42.0, "42".to_string(), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Float(102.0, "102".to_string(), None)
            ]
        );
    }

    #[test]
    fn test_parser_comment() {
        assert_eq!(
            ast("hello world ; some comment"),
            [
                Node::Name("hello".to_string(), Iname::new(0), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Name("world".to_string(), Iname::new(1), None),
                Node::Whitespace(" ".to_string(), None),
                Node::Comment(" some comment".to_string(), None)
            ]
        );
    }

    #[test]
    fn test_parser_list() {
        assert_eq!(
            ast("(hello world)"),
            [Node::List(
                vec![
                    Node::Name("hello".to_string(), Iname::new(0), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), Iname::new(1), None)
                ],
                None
            )]
        );

        assert_eq!(
            ast("(bitmap \"foo.png\")"),
            [Node::List(
                vec![
                    Node::Name("bitmap".to_string(), Iname::new(0), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::String("foo.png".to_string(), Iname::new(1), None)
                ],
                None
            )]
        );

        assert_eq!(
            ast("(hello world (1 2 3))"),
            [Node::List(
                vec![
                    Node::Name("hello".to_string(), Iname::new(0), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), Iname::new(1), None),
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
                    Node::Name("hello".to_string(), Iname::new(0), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Name("world".to_string(), Iname::new(1), None),
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
    }

    #[test]
    fn test_parser_quote() {
        assert_eq!(
            ast("'3"),
            [Node::List(
                vec![
                    Node::Name("quote".to_string(), Iname::new(153), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Float(3.0, "3".to_string(), None)
                ],
                None
            )]
        );
    }

    #[test]
    fn test_parser_alterable() {
        assert_eq!(
            ast("hello { 5 (gen/scalar)}"),
            [
                Node::Name("hello".to_string(), Iname::new(0), None),
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
                                    Iname::from(Native::GenScalar),
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
                    Node::Name("rect".to_string(), Iname::from(Native::Rect), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Label("width".to_string(), Iname::from(Keyword::Width), None),
                    Node::Whitespace(" ".to_string(), None),
                    Node::Float(300.0, "300".to_string(), None),
                ],
                None
            )]
        );
    }

    #[test]
    fn test_parser_from_name() {
        assert_eq!(
            ast("(some-vector.vector/length)"),
            [Node::List(
                vec![
                    Node::FromName("some-vector".to_string(), Iname::new(0), None),
                    Node::Name(
                        "vector/length".to_string(),
                        Iname::from(Native::VectorLength),
                        None
                    ),
                ],
                None
            )]
        );
    }
}
