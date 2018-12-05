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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Token<'a> {
    BackQuote,
    Colon,
    Comment(&'a str),
    CurlyBracketEnd,
    CurlyBracketStart,
    DoubleQuote,
    Name(&'a str),
    Number(&'a str),
    ParenEnd,
    ParenStart,
    Quote,
    SquareBracketEnd,
    SquareBracketStart,
    Whitespace(&'a str),
    End,
}

pub fn tokenize(s: &str) -> SenResult<Vec<Token>> {
    let mut lex = Lexer::new(s);
    let mut res = Vec::new();

    loop {
        match lex.eat_token()? {
            Token::End => break,
            tok => res.push(tok),
        }
    }

    Ok(res)
}

struct Lexer<'a> {
    input: &'a str,
    cur_pos: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &str) -> Lexer {
        Lexer{
            input: input,
            cur_pos: 0,
        }
    }

    pub fn eat_token(&mut self) -> SenResult<Token<'a>> {
        let mut chars = self.input.char_indices();

        while let Some((ind, ch)) = chars.next() {
            let res = match ch {
                '(' => Ok((Token::ParenStart, 1)),
                ')' => Ok((Token::ParenEnd, 1)),
                '[' => Ok((Token::SquareBracketStart, 1)),
                ']' => Ok((Token::SquareBracketEnd, 1)),
                '{' => Ok((Token::CurlyBracketStart, 1)),
                '}' => Ok((Token::CurlyBracketEnd, 1)),
                ':' => Ok((Token::Colon, 1)),
                '\'' => Ok((Token::Quote, 1)),
                '"' => Ok((Token::DoubleQuote, 1)),
                '`' => Ok((Token::BackQuote, 1)),
                ';' => eat_comment(&self.input[ind..]),
                '-' | '0' ... '9' => eat_number(&self.input[ind..]),
                ch if ch.is_whitespace() => eat_whitespace(&self.input[ind..]),
                _ if is_name(ch) => eat_name(&self.input[ind..]),
                ch => Err(SenError::ParserInvalidChar(ch))
            };

            let (tok, size) = match res {
                Ok(v) => v,
                Err(kind) => return Err(kind)
            };

            self.cur_pos += size as u32;
            self.input = &self.input[ind + size..];

            return Ok(tok);
        }

        self.input = &self.input[..0];
        Ok(Token::End)
    }
}

fn is_name(ch: char) -> bool {
    ch.is_alphanumeric() || is_symbol(ch)
}

fn is_symbol(ch: char) -> bool {
    match ch {
        '+' | '-' | '*' | '/' | '=' | '!' | '@' |
        '#' | '$' | '%' | '^' | '&' | '<' | '>' |
        '?' => true,
        _ => false
    }
}

fn eat_name(input: &str) -> SenResult<(Token, usize)> {
    for (ind, ch) in input.char_indices() {
        if !is_name(ch) {
            return Ok((Token::Name(&input[..ind]), ind));
        }
    }

    Ok((Token::Name(input), input.len()))
}

fn eat_whitespace(input: &str) -> SenResult<(Token, usize)> {
    for (ind, ch) in input.char_indices() {
        if !ch.is_whitespace() {
            return Ok((Token::Whitespace(&input[..ind]), ind));
        }
    }

    Ok((Token::Whitespace(input), input.len()))
}

fn eat_number(input: &str) -> SenResult<(Token, usize)> {
    let mut digits = false;
    let mut dot = false;
    let mut size = input.len();

    let (prefix_offset, rest) = if input.starts_with('-') {
        match input[1..].chars().next() {
            Some(ch) if ch.is_digit(10) => (1, &input[1..]),
            // Actually a name beginning with '-' rather a number
            _ => return eat_name(input)
        }
    } else {
        (0, input)
    };

    for (ind, ch) in rest.char_indices() {
        match ch {
            '.' => {
                if dot {
                    return Err(SenError::ParserInvalidLiteral);
                }
                dot = true;
            }
            _ if ch.is_digit(10) => {
                digits = true;
            }
            _ => {
                size = prefix_offset + ind;
                break;
            }
        }
    }

    if !digits {
        Err(SenError::ParserInvalidLiteral)
    } else {
        Ok((Token::Number(&input[..size]), size))
    }
}

fn eat_comment(input: &str) -> SenResult<(Token, usize)> {
    let rest = &input[1..];     // remove the first character (;)
    let mut size = rest.len();

    for (ind, ch) in rest.char_indices() {
        match ch {
            '\n' =>{
                size = ind;
                break;
            }
            _ => {
                continue;
            }
        }
    }
    Ok((Token::Comment(&rest[..size]), size + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        assert_eq!(tokenize("()").unwrap(),
                   [Token::ParenStart,
                    Token::ParenEnd]);

        assert_eq!(tokenize("( )").unwrap(),
                   [Token::ParenStart,
                    Token::Whitespace(" "),
                    Token::ParenEnd]);

        assert_eq!(tokenize("[]").unwrap(),
                   [Token::SquareBracketStart,
                    Token::SquareBracketEnd]);

        assert_eq!(tokenize("{}").unwrap(),
                   [Token::CurlyBracketStart,
                    Token::CurlyBracketEnd]);

        assert_eq!(tokenize("'(1)").unwrap(),
                   [Token::Quote,
                    Token::ParenStart,
                    Token::Number("1"),
                    Token::ParenEnd]);

        assert_eq!(tokenize("5").unwrap(),
                   [Token::Number("5")]);
        assert_eq!(tokenize("-3").unwrap(),
                   [Token::Number("-3")]);
        assert_eq!(tokenize("3.14").unwrap(),
                   [Token::Number("3.14")]);
        assert_eq!(tokenize("-0.34").unwrap(),
                   [Token::Number("-0.34")]);

        assert_eq!(tokenize("1 foo 3").unwrap(),
                   [Token::Number("1"),
                    Token::Whitespace(" "),
                    Token::Name("foo"),
                    Token::Whitespace(" "),
                    Token::Number("3")]);

        assert_eq!(tokenize("hello\nworld").unwrap(),
                   [Token::Name("hello"),
                    Token::Whitespace("\n"),
                    Token::Name("world")]);

        assert_eq!(tokenize("hello ; some comment").unwrap(),
                   [Token::Name("hello"),
                    Token::Whitespace(" "),
                    Token::Comment(" some comment")]);

        assert_eq!(tokenize("hello ; some comment\n(valid)").unwrap(),
                   [Token::Name("hello"),
                    Token::Whitespace(" "),
                    Token::Comment(" some comment"),
                    Token::Whitespace("\n"),
                    Token::ParenStart,
                    Token::Name("valid"),
                    Token::ParenEnd]);

        assert_eq!(tokenize("{ 23 (gen/scalar min: 3 max: 100)}").unwrap(),
                   [Token::CurlyBracketStart,
                    Token::Whitespace(" "),
                    Token::Number("23"),
                    Token::Whitespace(" "),
                    Token::ParenStart,
                    Token::Name("gen/scalar"),
                    Token::Whitespace(" "),
                    Token::Name("min"),
                    Token::Colon,
                    Token::Whitespace(" "),
                    Token::Number("3"),
                    Token::Whitespace(" "),
                    Token::Name("max"),
                    Token::Colon,
                    Token::Whitespace(" "),
                    Token::Number("100"),
                    Token::ParenEnd,
                    Token::CurlyBracketEnd]);
    }
}
