import Token from '../../src/lang/token';
import TokenType from '../../src/lang/tokentype';

describe('token', () => {

  it('should be created with a type and an optional value', () => {
    let t = new Token(TokenType.INT, 4);
    expect(t.value).to.equal(4);
    expect(t.type).to.equal(TokenType.INT);

    t = new Token(TokenType.UNKNOWN);
    expect(t.value).to.equal(undefined);
  });

  it('should get values for the constants', () => {
    expect(TokenType.UNKNOWN).to.equal(0);
  });
});

