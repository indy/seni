/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

import Interpreter from './Interpreter';
import Parser from './Parser';
import Unparser from './Unparser';
import Lexer from './Lexer';
import Compiler from './Compiler';
import Bind from './Bind';

const Runtime = {
  createEnv: () =>
    Bind.addClassicBindings(
      Bind.addSpecialDebugBindings(
        Bind.addSpecialBindings(
          Interpreter.getBasicEnv()))),

  buildFrontAst: form => {
    const tokensBox = Lexer.tokenise(form);
    if (tokensBox.error) {
      return { error: tokensBox.error };
    }
    const astBox = Parser.parse(tokensBox.tokens);
    if (astBox.error) {
      return { error: astBox.error };
    }
    return { nodes: astBox.nodes };
  },

  unparse: (frontAst, genotype) =>
    Unparser.unparse(frontAst, genotype),

  compileBackAst: frontAst =>
    Compiler.compileBackAst(frontAst),

  evalAst: (env, ast, genotype) => {
    const simplifiedAsts = Compiler.compileWithGenotype(ast, genotype);

    // add all of the define expressions to the env
    const [env1, res1, error1] = simplifiedAsts.
          filter(Interpreter.isDefineExpression).
          reduce(([e, _, error], form) => {
            if (error !== Interpreter.NO_ERROR) {
              return [e, _, error];
            }
            return Interpreter.evaluate(e, form);
          }, [env, false, Interpreter.NO_ERROR]);

    if (error1 !== Interpreter.NO_ERROR) {
      return [env1, res1, error1];
    }
    // a[0] === the new env returned by the interpreter

    // now evaluate all of the non-define expressions
    return simplifiedAsts.
      filter(s => !Interpreter.isDefineExpression(s)).
      reduce(([env2, res2, error2], form) => {
        if (error2 !== Interpreter.NO_ERROR) {
          return [env2, res2, error2];
        }
        return Interpreter.evaluate(env2, form);
      }, [env1, res1, Interpreter.NO_ERROR]);
  }
};

export default Runtime;
