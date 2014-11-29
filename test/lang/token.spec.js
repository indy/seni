import {
  Token,
  TokenType
} from '../../src/lang/token';

describe('token', function () {

  it('should be created with a type and an optional value', function () {
    let t = new Token(TokenType.INT, 4);
    expect(t.getValue()).toEqual(4);
    expect(t.getType()).toEqual(TokenType.INT);

    t = new Token(TokenType.UNKNOWN);
    expect(t.getValue()).toEqual(undefined);
  });

  it('should get values for the constants', function() {
    expect(TokenType.UNKNOWN).toEqual(0);
  });
});
