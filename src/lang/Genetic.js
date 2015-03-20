import NodeType from './NodeType';
import Compiler from './Compiler';
import Interpreter from './Interpreter';
import Bind from '../seni/Bind';
import SeedRandom from '../seni/SeedRandom';

function buildTraitFromNode(node, genes) {
  if(node.type === NodeType.LIST) {
    node.children.map((child) => buildTraitFromNode(child, genes));
  }

  if(node.alterable === true) {
    // expect a form in the parameterAST
    let ast;
    if(node.parameterAST.length) {
      // assuming that there aren't any nested square brackets
      ast = Compiler.compile(node.parameterAST);
    } else {
      // this is to allow code like (+ 2 [2])
      // which should behave as if there were no square brackets
      // todo: implement identity in this context
      ast = { forms: [['identity', {value: node.value}]]};
    }

    let gene = {initialValue: node.value,
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
  return Immutable.Map({value: finalResult});
}

let Genetic = {

  buildTraits: function(ast) {
    const traits = [];
    ast.map((node) => buildTraitFromNode(node, traits));
    return traits;
  },

  createGenotypeFromInitialValues: function(traits) {
    const genotype = traits.map((g) => Immutable.Map({value: g.initialValue}));
    return Immutable.List(genotype);
  },

  createGenotypeFromTraits: function(traits, seed) {

    const rng = SeedRandom.buildUnsigned(seed);
    const env = Bind.addBracketBindings(Interpreter.getBasicEnv(), rng);

    // env is the environment used to evaluate the bracketed forms
    const genotype = traits.map((trait) => buildGenoFromTrait(trait, env));

    return Immutable.List(genotype);
  }

};


export default Genetic;
