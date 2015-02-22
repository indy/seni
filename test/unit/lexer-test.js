import Lexer from '../../src/lang/lexer';
import TokenType from '../../src/lang/tokentype';


describe('Lexer', () => {

  it('should error handle', () => {
    // '|' is a character that currently isn't recognised by seni

    let q = Lexer.tokenise('|');
    expect(q.length).to.equal(1);
    expect(q[0].type).to.equal(TokenType.UNKNOWN);
    expect(q[0].value).to.equal('|');

    // if an illegal character is found in any part of the input, only that
    // character will be returned as the result of the lexing operation
    //
    q = Lexer.tokenise('(foo bar baz) | ');
    expect(q.length).to.equal(1);
    expect(q[0].type).to.equal(TokenType.UNKNOWN);
    expect(q[0].value).to.equal('|');
  });

  it('should tokenise strings', () => {
    let q = Lexer.tokenise('(go 42 3.14)');
    expect(q.length).to.equal(5);
    expect(q[0].type).to.equal(TokenType.LIST_START);
    expect(q[1].type).to.equal(TokenType.NAME);
    expect(q[2].type).to.equal(TokenType.INT);
    expect(q[3].type).to.equal(TokenType.FLOAT);
    expect(q[4].type).to.equal(TokenType.LIST_END);
  });

  it('should tokenise strings 2', () => {
    let q = Lexer.tokenise('(go [\"hi\"] \'SOMETHING)');
    expect(q.length).to.equal(8);
    expect(q[0].type).to.equal(TokenType.LIST_START);
    expect(q[1].type).to.equal(TokenType.NAME);
    expect(q[2].type).to.equal(TokenType.BRACKET_START);
    expect(q[3].type).to.equal(TokenType.STRING);
    expect(q[4].type).to.equal(TokenType.BRACKET_END);
    expect(q[5].type).to.equal(TokenType.QUOTE_ABBREVIATION);
    expect(q[6].type).to.equal(TokenType.NAME);
    expect(q[7].type).to.equal(TokenType.LIST_END);
  });

  it('should tokenise labeled function invocations', () => {
    let q = Lexer.tokenise('(go arg1: 42)');
    expect(q.length).to.equal(5);
    expect(q[0].type).to.equal(TokenType.LIST_START);
    expect(q[1].type).to.equal(TokenType.NAME);
    expect(q[2].type).to.equal(TokenType.LABEL);
    expect(q[3].type).to.equal(TokenType.INT);
    expect(q[4].type).to.equal(TokenType.LIST_END);
  });

  it('should recognise comments', () => {
    let q = Lexer.tokenise(';(go arg1: 42)');
    expect(q.length).to.equal(1);
    expect(q[0].type).to.equal(TokenType.COMMENT);

    q = Lexer.tokenise(';(go arg1: 42)\n(go arg1: 42)');
    expect(q.length).to.equal(6);
    expect(q[0].type).to.equal(TokenType.COMMENT);
    expect(q[1].type).to.equal(TokenType.LIST_START);
    expect(q[2].type).to.equal(TokenType.NAME);
    expect(q[3].type).to.equal(TokenType.LABEL);
    expect(q[4].type).to.equal(TokenType.INT);
    expect(q[5].type).to.equal(TokenType.LIST_END);
  });
});
