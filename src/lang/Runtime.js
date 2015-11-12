/*
 *  Seni
 *  Copyright (C) 2015  Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

import Interpreter from './Interpreter';
import Parser from './Parser';
import Lexer from './Lexer';
import Compiler from './Compiler';
import Genetic from './Genetic';

const Runtime = {
  createEnv: function() {
    return Interpreter.getBasicEnv();
  },

  buildAst: function(form) {
    const tokensBox = Lexer.tokenise(form);
    if (tokensBox.error) {
      console.log(tokensBox.error);
      return false;
    }
    const astBox = Parser.parse(tokensBox.tokens);
    if (astBox.error) {
      // some sort of error occurred
      console.log(astBox.error);
      return false;
    }

    const ast = Genetic.expandASTForAlterableChildren(astBox.nodes);

    return ast;
  },

  evalAst: function(env, ast, genotype) {
    const simplifiedAsts = Compiler.compile(ast, genotype);

    // add all of the define expressions to the env
    const [_env, _res] = simplifiedAsts.
            filter(Interpreter.isDefineExpression).
            reduce((a, b) => Interpreter.evaluate(a[0], b), [env, false]);
    // a[0] === the new env returned by the interpreter

    // now evaluate all of the non-define expressions
    return simplifiedAsts.
      filter(s => !Interpreter.isDefineExpression(s)).
      reduce((a, b) => Interpreter.evaluate(a[0], b), [_env, _res]);
  }
};

export default Runtime;
