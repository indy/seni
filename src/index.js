import Another from './another';
import TokenType from './lang/tokentype';
import Token from './lang/token';

const MyLibrary = {
  anotherFn: Another.anotherFn,
  mainFn() {
    let t = TokenType.INT;
    let tt = new Token(2, 3);
    let r = 'hello ' + t + ' ' + tt.type;
    r = 'hello';
    //console.log(r);
    return r;
  }
};

export default MyLibrary;
