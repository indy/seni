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
  if (node.alterable === true) {
    // expect a form in the parameterAST
    let simplifiedAst, initialValue;

    if(node.type === NodeType.LIST) {
      initialValue = Compiler.compileListNode(node);
    } else {
      initialValue = node.value;
    }

    if (node.parameterAST.length) {
      // assuming that there aren't any nested square brackets
      simplifiedAst = Compiler.compile(node.parameterAST);
    } else {
      // this is to allow code like (+ 2 [2])
      // which should behave as if there were no square brackets
      // todo: implement identity in this context
      simplifiedAst = [['identity', {value: initialValue}]];
    }

    const gene = {initialValue, simplifiedAst};
    genes.push(gene);  // mutate the genes
  }

  if (node.type === NodeType.LIST) {
    node.children.map(child => buildTraitFromNode(child, genes));
  }
}

function buildGenoFromTrait(trait, env) {
  const simplifiedAst = trait.simplifiedAst;
  // evaluate all of the forms, returning the final result
  const evalRes = simplifiedAst.reduce((a, b) => Interpreter.evaluate(a[0], b),
                                       [env, false]);
  // a[0] === the new env returned by the interpreter

  const finalResult = evalRes[1];
  return new Immutable.Map({value: finalResult});
}

function randomCrossover(genotypeA, genotypeB) {
  // todo: assert that both genotypes have the same length

  let crossoverIndex = Number.parseInt(Math.random() * genotypeA.size, 10);
  console.log('crossoverIndex', crossoverIndex);

  let spliceA = genotypeA.slice(0, crossoverIndex);
  let spliceB = genotypeB.slice(crossoverIndex, genotypeB.size);

  return spliceA.concat(spliceB);
}

const Genetic = {

  buildTraits: function(ast) {
    const traits = [];
    ast.map(node => buildTraitFromNode(node, traits));
    return traits;
  },

  createGenotypeFromInitialValues: function(traits) {
    const geno = traits.map(g => new Immutable.Map({value: g.initialValue}));
    return new Immutable.List(geno);
  },

  createGenotypeFromTraits: function(traits, seed) {
    const rng = SeedRandom.buildUnsigned(seed);
    const env = Bind.addBracketBindings(Interpreter.getBasicEnv(), rng);

    // env is the environment used to evaluate the bracketed forms
    const genotype = traits.map(trait => buildGenoFromTrait(trait, env));

    return new Immutable.List(genotype);
  },

  nextGeneration: function(genotypes, populationSize) {
    // a silly mod method for creating the latest generation
    let i, newGenotypes = [];

    // the chosen genotypes survive into the next generation
    for(i = 0; i < genotypes.length; i++) {
      newGenotypes[i] = genotypes[i];
    }

    for(i = genotypes.length; i < populationSize; i++) {
      let idxA = Number.parseInt(Math.random() * genotypes.length, 10);
      let idxB = Number.parseInt(Math.random() * genotypes.length, 10);

      // try not to use the same genotype for both a and b
      const retryCount = 10;
      for(let retry = 0; retry < retryCount; retry++) {
        if(idxB === idxA) {
          idxB = Number.parseInt(Math.random() * genotypes.length, 10);
        } else {
          break;
        }
      }

      let genotypeA = genotypes[idxA];
      let genotypeB = genotypes[idxB];

      console.log('crossover indices: ', idxA, idxB);
      newGenotypes.push(randomCrossover(genotypeA, genotypeB));
    }
    return newGenotypes;
  }
};

export default Genetic;
