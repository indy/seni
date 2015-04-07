/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import NodeType from './NodeType';
import Compiler from './Compiler';
import Interpreter from './Interpreter';
import Bind from '../seni/Bind';
import SeedRandom from '../seni/SeedRandom';
import Immutable from 'immutable';

function buildTraitFromNode(node, genes) {
  if (node.type === NodeType.LIST) {
    node.children.map((child) => buildTraitFromNode(child, genes));
  }

  if (node.alterable === true) {
    // expect a form in the parameterAST
    let ast;
    if (node.parameterAST.length) {
      // assuming that there aren't any nested square brackets
      ast = Compiler.compile(node.parameterAST);
    } else {
      // this is to allow code like (+ 2 [2])
      // which should behave as if there were no square brackets
      // todo: implement identity in this context
      ast = {forms: [['identity', {value: node.value}]]};
    }

    const gene = {initialValue: node.value,
                  ast: ast};
    genes.push(gene);  // mutate the genes
  }
}

function buildGenoFromTrait(trait, env) {
  const forms = trait.ast.forms;
  // evaluate all of the forms, returning the final result
  const evalRes = forms.reduce(([e, r], b) => {
    return Interpreter.evaluate(e, b);
  }, [env, false]);

  const finalResult = evalRes[1];
  return new Immutable.Map({value: finalResult});
}

const Genetic = {

  buildTraits: function(ast) {
    const traits = [];
    ast.map((node) => buildTraitFromNode(node, traits));
    return traits;
  },

  createGenotypeFromInitialValues: function(traits) {
    const genotype = traits.map((g) =>
                                new Immutable.Map({value: g.initialValue}));
    return new Immutable.List(genotype);
  },

  createGenotypeFromTraits: function(traits, seed) {

    const rng = SeedRandom.buildUnsigned(seed);
    const env = Bind.addBracketBindings(Interpreter.getBasicEnv(), rng);

    // env is the environment used to evaluate the bracketed forms
    const genotype = traits.map((trait) => buildGenoFromTrait(trait, env));

    return new Immutable.List(genotype);
  }

};

export default Genetic;
