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

import Interpreter from '../../app/js/lang/Interpreter';
import Parser from '../../app/js/lang/Parser';
import Lexer from '../../app/js/lang/Lexer';
import Compiler from '../../app/js/lang/Compiler';
import Genetic from '../../app/js/lang/Genetic';
import Bind from '../../app/js/lang/Bind';


export function buildEnv() {
  return Bind.addClassicBindings(
    Bind.addSpecialBindings(
      Interpreter.getBasicEnv()));
}

export function evalForm(env, form) {
  const ts = Lexer.tokenise(form).tokens;
  const ast = Parser.parse(ts).nodes;
  const traits = Genetic.buildTraits(ast);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);
  const backAst = Compiler.compileBackAst(ast);
  const astList = Compiler.compileWithGenotype(backAst, genotype);

  return astList.reduce(([e, res, err], ast2) => Interpreter.evaluate(e, ast2),
                        [env, undefined, Interpreter.NO_ERROR]);
}
