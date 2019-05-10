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

use crate::colour::ColourFormat;
use crate::error::Error;
use crate::gene::{Gene, Genotype};
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::parser::{parse, Node, NodeMeta, WordLut};
use crate::result::Result;

pub fn unparse(source: &str, genotype: &mut Genotype) -> Result<String> {
    let (ast, word_lut) = parse(source)?;
    let mut s: String = "".to_string();

    for n in &ast {
        unparse_ast_node(&mut s, &word_lut, n, genotype)?;
    }

    Ok(s)
}

pub fn simplified_unparse(source: &str) -> Result<String> {
    let (ast, word_lut) = parse(source)?;
    let mut s: String = "".to_string();

    for n in &ast {
        simplified_unparse_ast_node(&mut s, &word_lut, n)?;
    }

    Ok(s)
}

fn unparse_ast_node_alterable(
    cursor: &mut String,
    word_lut: &WordLut,
    ast: &Node,
    genotype: &mut Genotype,
    meta: &Option<NodeMeta>,
) -> Result<()> {
    cursor.push_str("{");

    if let Some(meta) = meta {
        for n in &meta.parameter_prefix {
            unparse_ast_node(cursor, word_lut, n, genotype)?;
        }

        let s = match ast {
            Node::Vector(_, _) => unparse_alterable_vector(&ast, genotype, word_lut)?,
            _ => format_var_value(&ast, genotype, word_lut)?,
        };
        cursor.push_str(&s);

        for n in &meta.parameter_ast {
            unparse_ast_node(cursor, word_lut, n, genotype)?;
        }
    }

    cursor.push_str("}");

    Ok(())
}

fn unparse_ast_node(
    cursor: &mut String,
    word_lut: &WordLut,
    ast: &Node,
    genotype: &mut Genotype,
) -> Result<()> {
    match ast {
        // todo: this is ugly, is there a cleaner way?
        Node::List(_, ref meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::Vector(_, meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::Float(_, _, meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::Name(_, _, meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::Label(_, _, meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::String(_, _, meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::Whitespace(_, meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::Comment(_, meta) if meta.is_some() => {
            unparse_ast_node_alterable(cursor, word_lut, ast, genotype, &meta)
        }
        Node::List(ns, _) => {
            if let Some(idx) = index_of_quote_keyword(&ns) {
                // rather than outputing: (quote (1 2 3))
                // we want: '(1 2 3)
                //
                cursor.push_str("'");

                // + 2 == skip past the quote name and the whitespace afterwards
                let nodes_after_quote = &ns[(idx + 2)..];

                for n in nodes_after_quote {
                    unparse_ast_node(cursor, word_lut, n, genotype)?
                }
            } else {
                cursor.push_str("(");
                for n in ns {
                    unparse_ast_node(cursor, word_lut, n, genotype)?
                }
                cursor.push_str(")");
            }
            Ok(())
        }
        Node::Vector(ns, _) => {
            cursor.push_str("[");
            for n in ns {
                unparse_ast_node(cursor, word_lut, n, genotype)?
            }
            cursor.push_str("]");
            Ok(())
        }
        _ => {
            let s = format_node_value(ast)?;
            cursor.push_str(&s);
            Ok(())
        }
    }
}

fn simplified_unparse_ast_node(cursor: &mut String, word_lut: &WordLut, ast: &Node) -> Result<()> {
    match ast {
        Node::List(ns, _) => {
            if let Some(idx) = index_of_quote_keyword(&ns) {
                // rather than outputing: (quote (1 2 3))
                // we want: '(1 2 3)
                //
                cursor.push_str("'");

                // + 2 == skip past the quote name and the whitespace afterwards
                let nodes_after_quote = &ns[(idx + 2)..];

                for n in nodes_after_quote {
                    simplified_unparse_ast_node(cursor, word_lut, n)?
                }
            } else {
                cursor.push_str("(");
                for n in ns {
                    simplified_unparse_ast_node(cursor, word_lut, n)?
                }
                cursor.push_str(")");
            }
            Ok(())
        }
        Node::Vector(ns, _) => {
            cursor.push_str("[");
            for n in ns {
                simplified_unparse_ast_node(cursor, word_lut, n)?
            }
            cursor.push_str("]");
            Ok(())
        }
        _ => {
            let s = format_node_value(ast)?;
            cursor.push_str(&s);
            Ok(())
        }
    }
}

fn index_of_quote_keyword(ns: &[Node]) -> Option<usize> {
    for (i, n) in ns.iter().enumerate() {
        if let Node::Name(_, iname, _) = n {
            if *iname == Iname::from(Keyword::Quote) {
                return Some(i);
            }
        }
    }
    None
}

fn format_node_value(node: &Node) -> Result<String> {
    match node {
        Node::List(_, _) => Err(Error::Unparser("Node::List ???".to_string())),
        Node::Vector(_, _) => Err(Error::Unparser("Node::Vector ???".to_string())),
        Node::Float(_, s, _) => Ok(s.to_string()),
        Node::Name(s, _, _) => Ok(s.to_string()),
        Node::Label(s, _, _) => Ok(s.to_string() + ":"),
        Node::String(s, _, _) => Ok("\"".to_owned() + &s.to_string() + "\""),
        Node::Whitespace(s, _) => Ok(s.to_string()),
        Node::Comment(s, _) => Ok(";".to_owned() + &s.to_string()),
    }
}

fn count_decimals(s: &str) -> usize {
    if let Some(index) = s.find('.') {
        s.len() - index - 1
    } else {
        0
    }
}

fn format_var_value(node: &Node, genotype: &mut Genotype, word_lut: &WordLut) -> Result<String> {
    let gene = &genotype.genes[genotype.current_gene_index];
    genotype.current_gene_index += 1;

    match gene {
        Gene::Float(f) => {
            if let Node::Float(_, s, _) = node {
                let num_decimals = count_decimals(s);
                Ok(format!("{:.*}", num_decimals, f))
            } else {
                Err(Error::Unparser(
                    "format_var_value Gene::Float not linked to Node::Float".to_string(),
                ))
            }
        }
        Gene::Name(i) => {
            if let Some(s) = word_lut.get_string_from_name(*i) {
                Ok(s.to_string())
            } else {
                dbg!(*i);
                Err(Error::Unparser(
                    "format_var_value Gene::Name iname has no string".to_string(),
                ))
            }
        }
        Gene::Colour(c) => match c.format {
            ColourFormat::Rgb => Ok(format!(
                "(col/rgb r: {:.*} g: {:.*} b: {:.*} alpha: {:.*})",
                2, c.e0, 2, c.e1, 2, c.e2, 2, c.e3
            )),
            ColourFormat::Hsl => Ok(format!(
                "(col/hsl h: {:.*} s: {:.*} l: {:.*} alpha: {:.*})",
                2, c.e0, 2, c.e1, 2, c.e2, 2, c.e3
            )),
            ColourFormat::Hsluv => Ok(format!(
                "(col/hsluv h: {:.*} s: {:.*} l: {:.*} alpha: {:.*})",
                2, c.e0, 2, c.e1, 2, c.e2, 2, c.e3
            )),
            ColourFormat::Hsv => Ok(format!(
                "(col/hsv h: {:.*} s: {:.*} v: {:.*} alpha: {:.*})",
                2, c.e0, 2, c.e1, 2, c.e2, 2, c.e3
            )),
            ColourFormat::Lab => Ok(format!(
                "(col/lab l: {:.*} a: {:.*} b: {:.*} alpha: {:.*})",
                2, c.e0, 2, c.e1, 2, c.e2, 2, c.e3
            )),
        },
        Gene::V2D(x, y) => {
            let mut res = "[".to_string();

            // node is a vector
            if let Node::Vector(ns, _) = node {
                let mut used_x = false;
                for n in ns {
                    match n {
                        Node::Float(_, s, _) => {
                            let num_decimals = count_decimals(s);

                            if used_x {
                                res.push_str(&format!("{:.*}", num_decimals, y));
                            } else {
                                res.push_str(&format!("{:.*}", num_decimals, x));
                                used_x = true;
                            }
                        }
                        _ => {
                            let ff = format_node_value(n)?;
                            res.push_str(&ff);
                        }
                    }
                }
            }

            res.push_str("]");

            Ok(res)
        }
        _ => Err(Error::Unparser(
            "format_var_value: unsupported gene type".to_string(),
        )),
    }
}

fn unparse_alterable_vector(
    node: &Node,
    genotype: &mut Genotype,
    word_lut: &WordLut,
) -> Result<String> {
    if let Node::Vector(ns, _) = node {
        let mut res = "[".to_string();

        for n in ns {
            let s = match n {
                Node::Whitespace(_, _) | Node::Comment(_, _) => format_node_value(n)?,
                _ => format_var_value(n, genotype, word_lut)?,
            };
            res.push_str(&s);
        }
        res.push_str("]");

        Ok(res)
    } else {
        Err(Error::Unparser(
            "unparse_alterable_vector requires a Node::Vector".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trait_list::TraitList;

    fn ast_and_genotype(source: &str, seed: i32) -> Result<(Vec<Node>, Genotype)> {
        let (ast, word_lut) = parse(source)?;

        let trait_list = TraitList::compile(&ast, &word_lut)?;
        let genotype = Genotype::build_from_seed(&trait_list, seed)?;

        Ok((ast, genotype))
    }

    fn seeded_unparse_check(seed: i32, source: &str, expected: &str) {
        let (_ast, mut genotype) = ast_and_genotype(source, seed).unwrap();
        let res = unparse(source, &mut genotype).unwrap();
        assert_eq!(expected, res);
    }

    // for source that has no genotypes
    fn basic_unparse_check(source: &str) {
        seeded_unparse_check(0, source, source);
    }

    fn simplify_check(source: &str, expected: &str) {
        let res = simplified_unparse(source).unwrap();
        assert_eq!(expected, res);
    }

    #[test]
    fn test_count_decimals() {
        assert_eq!(count_decimals("12"), 0);
        assert_eq!(count_decimals("12.3"), 1);
        assert_eq!(count_decimals("12.34"), 2);
        assert_eq!(count_decimals("12.345"), 3);
        assert_eq!(count_decimals("12.3456"), 4);
        assert_eq!(count_decimals("56312.3456"), 4);
    }

    #[test]
    fn test_unparser_basics() {
        basic_unparse_check("fn");
        basic_unparse_check("a");
        basic_unparse_check("robocop");
        basic_unparse_check("a ; some comment \"here\"");
        basic_unparse_check("(fn (a b: 10) (+ b 20))");

        basic_unparse_check("12");
        basic_unparse_check("12.1");
        basic_unparse_check("12.12");
        basic_unparse_check("12.123");

        basic_unparse_check("(+ 1 2)");
        basic_unparse_check("(define aaa 543)");
        basic_unparse_check("(define aaa (+ 1 3 4))");

        basic_unparse_check("(define aaa '(1 3 4))");

        basic_unparse_check("(define aaa [1])");
        basic_unparse_check("(define aaa [1  ])");
        basic_unparse_check("(define aaa [  1  ])");
        basic_unparse_check("(define aaa [1 2 3 4])");

        basic_unparse_check("(define aaa 1.2) (define bbb 54) (define ccc 9.0909)");

        basic_unparse_check("(bitmap \"foo.png\")");
    }

    #[test]
    fn test_unparser_seeded() {
        seeded_unparse_check(
            975,
            "(+ 6 {3 (gen/int min: 1 max: 50)})",
            "(+ 6 {46 (gen/int min: 1 max: 50)})",
        );
        seeded_unparse_check(
            975,
            "{rainbow (gen/select from: col/procedural-fn-presets)}",
            "{transformers (gen/select from: col/procedural-fn-presets)}",
        );
        seeded_unparse_check(
            342,
            "[8 {3 (gen/int min: 0 max: 9)}]",
            "[8 {4 (gen/int min: 0 max: 9)}]",
        );

        seeded_unparse_check(
            764,
            "{3.45 (gen/scalar min: 0 max: 9)}",
            "{2.38 (gen/scalar min: 0 max: 9)}",
        );

        seeded_unparse_check(
            764,
            "{3.4 (gen/scalar min: 0 max: 9)}",
            "{2.4 (gen/scalar min: 0 max: 9)}",
        );

        seeded_unparse_check(
            764,
            "(col/rgb r: {0.4 (gen/scalar)} g: 0.1)",
            "(col/rgb r: {0.3 (gen/scalar)} g: 0.1)",
        );

        seeded_unparse_check(
            764,
            "{3 (gen/select from: '(4 5 6 7))}",
            "{5 (gen/select from: '(4 5 6 7))}",
        );

        seeded_unparse_check(
            764,
            "(rect position: [500 500] colour: red width: {120 (gen/int min: 80 max:
400)} height: {140 (gen/int min: 80 max: 670)}) (rect position: [500
500] colour: red width: {120 (gen/int min: 80 max: 400)} height: {140
(gen/int min: 80 max: 670)}) (rect position: [500 500] colour: red
width: {120 (gen/int min: 80 max: 400)} height: {140 (gen/int min: 80
max: 670)})",
            "(rect position: [500 500] colour: red width: {164 (gen/int min: 80 max:
400)} height: {562 (gen/int min: 80 max: 670)}) (rect position: [500
500] colour: red width: {289 (gen/int min: 80 max: 400)} height: {663
(gen/int min: 80 max: 670)}) (rect position: [500 500] colour: red
width: {210 (gen/int min: 80 max: 400)} height: {148 (gen/int min: 80
max: 670)})",
        );

        seeded_unparse_check(
            764,
            "{b (gen/select from: '(a b c))}",
            "{a (gen/select from: '(a b c))}",
        );

        seeded_unparse_check(
            764,
            "{(col/rgb r: 1 g: 0 b: 0.4 alpha: 1) (gen/col)}",
            "{(col/rgb r: 0.82 g: 0.65 b: 0.99 alpha: 0.26) (gen/col)}",
        );

        seeded_unparse_check(
            653,
            "{(col/rgb r: 1 g: 0 b: 0.4 alpha: 1) (gen/col alpha: 1)}",
            "{(col/rgb r: 0.78 g: 0.97 b: 0.89 alpha: 1.00) (gen/col alpha: 1)}",
        );
    }

    #[test]
    fn test_unparser_vectors() {
        seeded_unparse_check(
            653,
            "{[[1.00 2.00] [3.00 4.00]] (gen/2d)}",
            "{[[0.78 0.97] [0.89 0.11]] (gen/2d)}",
        );

        seeded_unparse_check(
            653,
            "{[[  1.00   2.00  ] [  3.00   4.00  ]] (gen/2d)}",
            "{[[  0.78   0.97  ] [  0.89   0.11  ]] (gen/2d)}",
        );

        seeded_unparse_check(
            653,
            "{[[10 20] [30 40]] (gen/2d min: 60 max: 70)}",
            "{[[68 70] [69 61]] (gen/2d min: 60 max: 70)}",
        );

        seeded_unparse_check(
            653,
            "{ [ [ 50.1 60.23 ] [ 70.456 80.7890 ]] (gen/2d min: 40 max: 90) }",
            "{ [ [ 79.1 88.54 ] [ 84.577 45.4872 ]] (gen/2d min: 40 max: 90) }",
        );
    }

    #[test]
    fn test_unparser_single_trait_vectors() {
        seeded_unparse_check(
            764,
            "{[10 20] (gen/stray-2d from: [10 20] by: [5 5])}",
            "{[8 23] (gen/stray-2d from: [10 20] by: [5 5])}",
        );
    }

    #[test]
    fn test_unparser_multiple_floats() {
        seeded_unparse_check(
            764,
            "{[0.977 0.416 0.171] (gen/scalar)}",
            "{[0.265 0.816 0.654] (gen/scalar)}",
        );
    }

    #[test]
    fn test_simplified_unparser() {
        simplify_check("(+ 1 1)", "(+ 1 1)");
        simplify_check("(+ 6 {3 (gen/int min: 1 max: 50)})", "(+ 6 3)");
        simplify_check(
            "(col/rgb r: {0.4 (gen/scalar)} g: 0.1)",
            "(col/rgb r: 0.4 g: 0.1)",
        );
        simplify_check("{b (gen/select from: '(a b c))}", "b");
        simplify_check(
            "{robocop (gen/select from: col/procedural-fn-presets)}",
            "robocop",
        );
        simplify_check("{50 (gen/stray from: 50 by: 20)}", "50");
    }
}