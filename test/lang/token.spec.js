import {Token} from '../../src/lang/token';

describe('token', function () {

  it('should be created with a type and an optional value', function () {
    let t = new Token(Token.INT, 4);
    expect(t.getValue()).toEqual(4);
    expect(t.getType()).toEqual(Token.INT);

    t = new Token(Token.UNKNOWN);
    expect(t.getValue()).toEqual(undefined);
  });
});
