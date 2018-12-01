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

use lexer::Token;


struct Gene {
    this_is_a_placeholder: i32,
}

struct NodeMeta<'a> {
    gene: Option<Gene>,
    parameter_ast: Vec<Node<'a>>,
    parameter_prefix: Vec<Node<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Node<'a> {
    List(Vec<Node<'a>>),
    Vector(Vec<Node<'a>>),
    Int(&'a str),
    Float(&'a str),
    Name(&'a str),
    Label(&'a str),
    String(&'a str),
    Whitespace(&'a str),
    Comment(&'a str),
    Fake,
}


pub fn parse(tokens: Vec<Token>) -> Vec<Node> {
    let mut res = Vec::new();

    let tok = tokens[0];
    let node = match tok {
        Token::Name(n) => Node::Name(n),
        _ => Node::Fake
    };

    res.push(node);

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::tokenize;

    fn ast(s: &str) -> Vec<Node> {
        let t = tokenize(s).unwrap();
        let n = parse(t);
        n
    }

    #[test]
    fn test_parser() {
        assert_eq!(ast("hello"), [Node::Name("hello")])
    }
}
