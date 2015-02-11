import {
  parse
} from 'lang/parser';

import {
  NodeType
} from 'lang/node';

import {
  Token,
  TokenType
} from 'lang/token';

export function main() {
  describe('parse', () => {

    it('should parse an int', () => {
      let ts = [new Token(TokenType.INT, 4)];

      let res = parse(ts).nodes;

      expect(res.length).toEqual(1);
      expect(res[0].type).toEqual(NodeType.INT);
      expect(res[0].value).toEqual(4);
    });

    it('should parse a float', () => {
      let ts = [new Token(TokenType.FLOAT, 3.14)];

      let res = parse(ts).nodes;

      expect(res.length).toEqual(1);
      expect(res[0].type).toEqual(NodeType.FLOAT);
      expect(res[0].value).toBeCloseTo(3.14);
    });

    it('should parse a name', () => {
      let ts = [new Token(TokenType.NAME, "cdr")];

      let res = parse(ts).nodes;

      expect(res.length).toEqual(1);
      expect(res[0].type).toEqual(NodeType.NAME);
      expect(res[0].value).toEqual("cdr");
    });

    it('should parse a string', () => {
      let ts = [new Token(TokenType.STRING, "hello world")];

      let res = parse(ts).nodes;

      expect(res.length).toEqual(1);
      expect(res[0].type).toEqual(NodeType.STRING);
      expect(res[0].value).toEqual("hello world");
    });

    it('should parse a boolean', () => {
      let ts = [new Token(TokenType.NAME, "true")];
      let res = parse(ts).nodes;
      expect(res.length).toEqual(1);
      expect(res[0].type).toEqual(NodeType.BOOLEAN);
      expect(res[0].value).toEqual('#t');

      ts = [new Token(TokenType.NAME, "false")];
      res = parse(ts).nodes;
      expect(res.length).toEqual(1);
      expect(res[0].type).toEqual(NodeType.BOOLEAN);
      expect(res[0].value).toEqual('#f');
    });

    it('should parse a list', () => {
      let ts = [new Token(TokenType.LIST_START),
                new Token(TokenType.INT, 4),
                new Token(TokenType.LIST_END)];

      let res = parse(ts).nodes;

      expect(res.length).toEqual(1);
      expect(res[0].type).toEqual(NodeType.LIST);
    });


    it('should error on a mismatched list (no closing pair)', () => {
      let ts = [new Token(TokenType.LIST_START),
                new Token(TokenType.INT, 4)];

      let r = parse(ts);
      expect(r.error).toBeDefined();
    });

    it('should error on a mismatched list (no opening pair)', () => {
      let ts = [new Token(TokenType.INT, 4),
                new Token(TokenType.LIST_END)];

      let r = parse(ts);
      expect(r.error).toBeDefined();
    });
    

    it('should parse a quoted form', () => {
      // '(2 3 4) => (quote (2 3 4))
      let ts = [new Token(TokenType.QUOTE_ABBREVIATION),
                new Token(TokenType.LIST_START),
                new Token(TokenType.INT, 2),
                new Token(TokenType.INT, 3),
                new Token(TokenType.INT, 4),
                new Token(TokenType.LIST_END)];

      let res = parse(ts).nodes;
      expect(res.length).toEqual(1);

      let lst = res[0];
      expect(lst.type).toEqual(NodeType.LIST);
      expect(lst.size()).toEqual(2);

      let quote = lst.getChild(0);
      expect(quote.type).toEqual(NodeType.NAME);
      expect(quote.value).toEqual("quote");

      let quotedList = lst.getChild(1);
      expect(quotedList.type).toEqual(NodeType.LIST);
      expect(quotedList.size()).toEqual(3);
    });

    it('should parse multiple lists', () => {
      let ts = [new Token(TokenType.LIST_START),
                new Token(TokenType.INT, 4),
                new Token(TokenType.LIST_END),
                new Token(TokenType.LIST_START),
                new Token(TokenType.INT, 45),
                new Token(TokenType.LIST_END),
                new Token(TokenType.LIST_START),
                new Token(TokenType.INT, 456),
                new Token(TokenType.LIST_END)];

      let res = parse(ts).nodes;

      expect(res.length).toEqual(3);
      expect(res[0].type).toEqual(NodeType.LIST);
      expect(res[1].type).toEqual(NodeType.LIST);
      expect(res[2].type).toEqual(NodeType.LIST);
    });

    it('should parse a bracket form', () => {
      let ts = [new Token(TokenType.BRACKET_START),
                new Token(TokenType.INT, 42),
                new Token(TokenType.LIST_START),
                new Token(TokenType.INT, 22),
                new Token(TokenType.INT, 88),
                new Token(TokenType.LIST_END),
                new Token(TokenType.BRACKET_END)];

      let r = parse(ts);
      let res = r.nodes;
      
      expect(res.length).toEqual(1);
      let alterableNode = res[0];
      expect(alterableNode.type).toEqual(NodeType.INT);
      expect(alterableNode.value).toEqual(42);
      expect(alterableNode.alterable).toEqual(true);

      let parameterNodes = alterableNode.parameterAST;
      expect(parameterNodes.length).toEqual(1);

      let params = parameterNodes[0];
      expect(params.type).toEqual(NodeType.LIST);
      
    });

  });
}
