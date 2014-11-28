import {Lexer} from '../../src/lang/lexer';

describe('Lexer', function () {
  var lexer;

  beforeEach(function () {
    lexer = new Lexer();
  });

  it('should double', function () {
    expect(lexer.doubler(3)).toEqual(6);
  });

  it('should double again', function () {
    expect(lexer.doubler(3)).toEqual(6);
  });

});
