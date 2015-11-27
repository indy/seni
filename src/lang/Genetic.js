/*
 *  Seni
 *  Copyright (C) 2015 Inderjit Gill <email@indy.io>
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

import NodeType from './NodeType';
import Compiler from './Compiler';
import Interpreter from './Interpreter';
import Bind from '../seni/Bind';
import PseudoRandom from '../seni/PseudoRandom';
import Immutable from 'immutable';

// the node will be in backAst form
function buildTraitFromNode(node, genes) {
  if (node.alterable === true) {

    // expect a form in the parameterAST
    let simplifiedAst, initialValue, compiledNodes;

    if(node.type === NodeType.LIST) {
      // wrap the node in an array and pass into Compiler.compileWithGenotype
      compiledNodes = Compiler.compileWithGenotype([node], null);
      initialValue = compiledNodes[0];
    } else {
      initialValue = node.value;
    }

    if (node.parameterAST.length) {
      simplifiedAst = Compiler.compileWithGenotype(node.parameterAST, null);
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

function buildGeneFromTrait(trait, env) {
  const simplifiedAst = trait.simplifiedAst;
  // evaluate all of the forms, returning the final result
  const evalRes = simplifiedAst.reduce((a, b) => Interpreter.evaluate(a[0], b),
                                       [env, false]);
  // evalRes[0] === the new env returned by the interpreter
  return evalRes[1];
}

function randomCrossover(genotypeA, genotypeB, mutationRate, traits, env) {
  // todo: assert that both genotypes have the same length

  let crossoverIndex = Number.parseInt(Math.random() * genotypeA.size, 10);
  console.log('randomCrossover index:', crossoverIndex, mutationRate);

  let spliceA = genotypeA.slice(0, crossoverIndex);
  let spliceB = genotypeB.slice(crossoverIndex, genotypeB.size);

  let childGenotype = spliceA.concat(spliceB);

  let i;
  for(i = 0; i < genotypeA.size; i++) {
    if(Math.random() < mutationRate) {
      // mutate this trait
      console.log('mutating gene ', i);
      childGenotype[i] = buildGeneFromTrait(traits[i], env);
    }
  }

  return new Immutable.List(childGenotype);
}

const Genetic = {

  buildTraits: function(ast) {
    const traits = [];
    ast.map(node => buildTraitFromNode(node, traits));
    return traits;
  },

  createGenotypeFromInitialValues: function(traits) {
    return new Immutable.List(traits.map(g => g.initialValue));
  },

  createGenotypeFromTraits: function(traits, seed) {
    const rng = PseudoRandom.buildUnsigned(seed);
    const env = Bind.addBracketBindings(Interpreter.getBasicEnv(), rng);

    // env is the environment used to evaluate the bracketed forms
    const genotype = traits.map(trait => buildGeneFromTrait(trait, env));

    return new Immutable.List(genotype);
  },

  nextGeneration: function(genotypes, populationSize, mutationRate, traits) {
    // a silly mod method for creating the latest generation
    let i, newGenotypes = [];
    const seed = 42;
    const rng = PseudoRandom.buildUnsigned(seed);
    const env = Bind.addBracketBindings(Interpreter.getBasicEnv(), rng);

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

      console.log('using genotype indices: ', idxA, idxB);

      let child = randomCrossover(genotypeA, genotypeB,
                                  mutationRate, traits, env);

      newGenotypes.push(child);
    }
    return newGenotypes;
  }
};

export default Genetic;
