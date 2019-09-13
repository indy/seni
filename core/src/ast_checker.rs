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

use std::collections::HashSet;
use crate::parser::{Node, WordLut};
use crate::error::Result;
use crate::iname::Iname;
use crate::keywords::Keyword;

pub fn check(ast: &[Node], _word_lut: &WordLut) -> Result<()> {
    let global_defines = find_global_defines(ast)?;
    Ok(())
}

fn find_global_defines(ast: &[Node]) -> Result<HashSet<Iname>> {
    let mut global_defines: HashSet<Iname> = HashSet::new();

    let iname_define = Iname::from(Keyword::Define);

    for node in ast {
        match node {
            Node::List(all_children, _) => {
                let children = semantic_children(all_children);
                if children.len() > 0 && children[0].is_name_with_iname(iname_define) {
                    add_global_decls(&mut global_defines, &children[1..]);
                }
            },
            _ => {}
        }
    }

    Ok(global_defines)
}

fn add_global_decls(global_defines: &mut HashSet<Iname>, nodes: &[&Node]) {
    let mut ns = nodes;

    // nodes is a list of nodes organised into binding/value pairs
    while ns.len() > 1 {
        match &ns[0] {
            Node::Name(_, iname, _) => {
                global_defines.insert(*iname);
            },
            Node::Vector(all_children, _) => {
                add_global_vector_decls(global_defines, all_children)
            },
            _ => {
            }
        };

        ns = &ns[2..];
    }
}

fn add_global_vector_decls(global_defines: &mut HashSet<Iname>, nodes: &[Node]) {
    for child in semantic_children(nodes) {
        match child {
            Node::Name(_, iname, _) => {
                global_defines.insert(*iname);
            },
            Node::Vector(all_children, _) => {
                add_global_vector_decls(global_defines, all_children)
            }
            _ => {}
        }
    }
}

fn semantic_children(nodes: &[Node]) -> Vec<&Node> {
    nodes.into_iter().filter(|c| c.is_semantic()).collect()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::parser::parse;

    /*
(define e 45)
(define [a b c d] [...])

--------------------------------------------------------------------------------
(+ 4 xxx)            <---- error: xxx not defined
--------------------------------------------------------------------------------
(+ 4 z)              <---- error: z not defined
--------------------------------------------------------------------------------
(fn (foo bar: 3)
    (+ q bar))       <---- error: q
--------------------------------------------------------------------------------
(fn (foo bar: 3)
    (define z 334)
    (+ z bar))

(+ z z)              <--- error: z not in scope
--------------------------------------------------------------------------------
(fn (foo bar: 3)
    (+ foo bar))     <---- error: foo is not a variable
*/

    fn checks_ok(s: &str) {
        let (ast, word_lut) = parse(s).unwrap();
        if let Ok(()) = check(&ast, &word_lut) {
        } else {
            assert!(false, "{}", s);
        }
    }

    #[test]
    fn test_check() {
        checks_ok("(+ 1 1)");
    }

    #[test]
    fn test_find_global_defines() {
        // single define
        {
            let (ast, _word_lut) = parse("(define a 45)").unwrap();
            let global_defines = find_global_defines(&ast).unwrap();

            assert_eq!(global_defines.len(), 1);
        }
        // multiple bindings in a single define statement
        {
            let (ast, _word_lut) = parse("(define a 45 b 56 c 67)").unwrap();
            let global_defines = find_global_defines(&ast).unwrap();

            assert_eq!(global_defines.len(), 3);
        }
        // vector destructuring
        {
            let (ast, _word_lut) = parse("(define [a b c d] [1 2 3 4])").unwrap();
            let global_defines = find_global_defines(&ast).unwrap();

            assert_eq!(global_defines.len(), 4);
        }
        // nested vector destructuring
        {
            let (ast, _word_lut) = parse("(define [[a b] [c d e]] whatever)").unwrap();
            let global_defines = find_global_defines(&ast).unwrap();

            assert_eq!(global_defines.len(), 5);
        }
    }

}
