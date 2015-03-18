/*jslint ignore:start*/

import Parser from '../../src/lang/Parser';
import Lexer from '../../src/lang/Lexer';
import Genetic from '../../src/lang/Genetic';

describe('genetic', () => {

  function simpleBuildTraits(form) {
    // assumes that the form will compile into a single list
    let ts = Lexer.tokenise(form).tokens;
    let ast = Parser.parse(ts).nodes;

    return Genetic.buildTraits(ast);
  }

  it('should build a traits array from an ast', () => {
    let res = simpleBuildTraits('(+ 3 [4 (inRange min: 0 max: 8)])');
    expect(res.length).to.equal(1);
    expect(res[0].initialValue).to.equal(4);
    // the ast should be in compiled form
    expect(res[0].ast.forms.length).to.equal(1);
    expect(res[0].ast.forms[0][0]).to.equal('inRange');
  });

  it('should default bracketed forms to have an identity function', () => {
    let res = simpleBuildTraits('(+ 2 [1])');
    expect(res.length).to.equal(1);
    expect(res[0].initialValue).to.equal(1);
    expect(res[0].ast.forms.length).to.equal(1);
    expect(res[0].ast.forms[0][0]).to.equal('identity');
  });

  it('should createGenotypeFromTraits', () => {
    let ts = Lexer.tokenise('(+ 2 [44])').tokens;
    let ast = Parser.parse(ts).nodes;

    let traits = Genetic.buildTraits(ast);

    let genotype = Genetic.createGenotypeFromTraits(traits, 100);

    expect(genotype.size).to.equal(1);
    expect(genotype.get(0).get('value')).to.equal(44);
  });


  it('should createGenotypeFromTraits 2', () => {
    let ts = Lexer.tokenise('(+ 2 [44 (inRange min: 10 max: 56)])').tokens;
    let ast = Parser.parse(ts).nodes;

    let traits = Genetic.buildTraits(ast);

    let genotype = Genetic.createGenotypeFromTraits(traits, 100);

    expect(genotype.size).to.equal(1);
    // the 11 is returned from an rng with a seed of 100
    expect(genotype.get(0).get('value')).to.equal(11);
  });

  it('should create the same genotype', () => {
    let ts = Lexer.tokenise('(+ 2 [44 (inRange min: 10 max: 56)])').tokens;
    let ast = Parser.parse(ts).nodes;

    let traits = Genetic.buildTraits(ast);

    let genotype = Genetic.createGenotypeFromTraits(traits, 33);
    expect(genotype.get(0).get('value')).to.equal(49);
    // the same seed should generate the same genotype
    genotype = Genetic.createGenotypeFromTraits(traits, 33);
    expect(genotype.get(0).get('value')).to.equal(49);
  });
});
