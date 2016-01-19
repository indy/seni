/*
    Seni
    Copyright (C) 2015 Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

import Parser from '../../app/js/lang/Parser';
import Lexer from '../../app/js/lang/Lexer';
import Compiler from '../../app/js/lang/Compiler';
import Genetic from '../../app/js/lang/Genetic';
import Node from '../../app/js/lang/Node';
import NodeList from '../../app/js/lang/NodeList';
import NodeVector from '../../app/js/lang/NodeVector';
import NodeType from '../../app/js/lang/NodeType';

import {expect} from 'chai';

describe('Compiler', () => {

  function compileAst(form) {
    const ts = Lexer.tokenise(form).tokens;
    const frontAst = Parser.parse(ts).nodes;
    const backAst = Compiler.compileBackAst(frontAst);

    const traits = Genetic.buildTraits(backAst);

    return [backAst, traits];
  }

  function compile(form) {
    const [backAst, traits] = compileAst(form);

    const genotype = Genetic.createGenotypeFromInitialValues(traits);

    const simplifiedAsts =  Compiler.compileWithGenotype(backAst, genotype);
    return simplifiedAsts[0];
  }

  function compileWithSeed(form, seed) {
    const [backAst, traits] = compileAst(form);

    const genotype = Genetic.createGenotypeFromTraits(traits, seed);

    const simplifiedAsts = Compiler.compileWithGenotype(backAst, genotype);
    return simplifiedAsts[0];
  }

  it('should compile a backAst', () => {
    // create a node
    function n(type, value) {
      switch (type) {
      case NodeType.LIST:
        return new NodeList();
      case NodeType.VECTOR:
        return new NodeVector();
      default:
        return new Node(type, value);
      }
    }

    let node, frontAst, backAst;

    // 2
    frontAst = [n(NodeType.INT, 2)];
    backAst = Compiler.compileBackAst(frontAst);
    expect(backAst.length).to.equal(1);
    expect(backAst[0].type).to.equal(NodeType.INT);
    expect(backAst[0].value).to.equal(2);

    // start with some whitespace which should be removed
    // ^_^_2
    frontAst = [n(NodeType.WHITESPACE, ' '),
                n(NodeType.WHITESPACE, ' '),
                n(NodeType.INT, 2)];
    backAst = Compiler.compileBackAst(frontAst);
    expect(backAst.length).to.equal(1);
    expect(backAst[0].type).to.equal(NodeType.INT);
    expect(backAst[0].value).to.equal(2);

    // (hello 3) ;; some comment
    node = n(NodeType.LIST, null);
    node.children = [n(NodeType.STRING, 'hello'),
                     n(NodeType.WHITESPACE, ' '),
                     n(NodeType.INT, 3)];
    frontAst = [node,
                n(NodeType.WHITESPACE, ' '),
                n(NodeType.COMMENT, 'some comment')];
    backAst = Compiler.compileBackAst(frontAst);
    expect(backAst.length).to.equal(1);
    expect(backAst[0].type).to.equal(NodeType.LIST);
    expect(backAst[0].children.length).to.equal(2);
    expect(backAst[0].children[0].value).to.equal('hello');
    expect(backAst[0].children[1].value).to.equal(3);


    // (hello (mult 5 6) 3) ;; some comment
    const node2 = n(NodeType.LIST, null);
    node2.children = [n(NodeType.STRING, 'mult'),
                      n(NodeType.WHITESPACE, ' '),
                      n(NodeType.INT, 5),
                      n(NodeType.WHITESPACE, ' '),
                      n(NodeType.INT, 6)];
    node = n(NodeType.LIST, null);
    node.children = [n(NodeType.STRING, 'hello'),
                     n(NodeType.WHITESPACE, ' '),
                     node2,
                     n(NodeType.WHITESPACE, ' '),
                     n(NodeType.INT, 3)];
    frontAst = [node,
                n(NodeType.WHITESPACE, ' '),
                n(NodeType.COMMENT, 'some comment')];
    backAst = Compiler.compileBackAst(frontAst);
    expect(backAst.length).to.equal(1);
    expect(backAst[0].type).to.equal(NodeType.LIST);
    expect(backAst[0].children.length).to.equal(3);
    expect(backAst[0].children[0].value).to.equal('hello');
    expect(backAst[0].children[1].type).to.equal(NodeType.LIST);
    const grandkids = backAst[0].children[1].children;
    expect(grandkids.length).to.equal(3);
    expect(grandkids[0].value).to.equal('mult');
    expect(grandkids[1].value).to.equal(5);
    expect(grandkids[2].value).to.equal(6);
    expect(backAst[0].children[2].value).to.equal(3);

    // [5 6]
    node = n(NodeType.VECTOR, null);
    node.children = [n(NodeType.INT, 5),
                     n(NodeType.WHITESPACE, ' '),
                     n(NodeType.INT, 6)];
    frontAst = [node];
    backAst = Compiler.compileBackAst(frontAst);
    expect(backAst.length).to.equal(1);
    expect(backAst[0].type).to.equal(NodeType.VECTOR);
    expect(backAst[0].children.length).to.equal(2);
    expect(backAst[0].children[0].value).to.equal(5);
    expect(backAst[0].children[0].type).to.equal(NodeType.INT);
    expect(backAst[0].children[1].value).to.equal(6);
    expect(backAst[0].children[1].type).to.equal(NodeType.INT);

    // [5 (int)]
  });

  it('should test required functions: genotype initial', () => {
    expect(compile('(* 2 {4})')).
      to.deep.equal(['*', 2, 4]);
  });

  it('should build a hash for the arguments', () => {
    expect(compile('(something monkey: 42)')).
      to.deep.equal(['something', {monkey: 42}]);
  });

  it('should test required functions: genotype', () => {
    expect(compileWithSeed('(+ 2 {44 (int min: 10 max: 56)})', 100)).
      to.deep.equal(['+', 2, 11]);

    expect(compileWithSeed('(+ 2 {44 (int min: 10 max: 56)})', 33)).
      to.deep.equal(['+', 2, 49]);
    expect(compileWithSeed('(+ 2 {44 (int min: 10 max: 56)})', 33)).
      to.deep.equal(['+', 2, 49]);

  });

  // correctly work with functions that have genotypes
  // in both name and arg positions
  it('should correctly apply genotypes to named functions', () => {
    const form = `({gogo (select from: (list 'shabba 'gogo 'hooha))}
                 foo: {44 (int min: 10 max: 56)})`;

    expect(compileWithSeed(form, 100)).to.deep.equal(['shabba', {'foo': 15}]);
  });

  it('should test plus', () => {
    expect(compileWithSeed('({- (testPlus)} 2 7)', 100)).
      to.deep.equal(['+', 2, 7]);
  });

  it('should bracket bind a random colour', () => {
    let res = (compileWithSeed('{(col/rgb r: 0.1 g: 0.2 b: 0.3) (col)}', 100));
    // res === ['col/rgb', {r: 0.122, g: 0.08, b: 0.40}]

    expect(res.length).to.equal(2);
    expect(res[0]).to.equal('col/rgb');

    const epsilon = 0.01;
    let colourValues = res[1];
    expect(colourValues.r).to.be.closeTo(0.0253, epsilon);
    expect(colourValues.g).to.be.closeTo(0.1226, epsilon);
    expect(colourValues.b).to.be.closeTo(0.0826, epsilon);
    expect(colourValues.alpha).to.be.closeTo(0.4031, epsilon);

    res = (compileWithSeed('{(col/rgb r: 0.1 g: 0.2 b: 0.3) (col alpha: 1)}',
                            100));
    // res === ['col/rgb', {r: 0.122, g: 0.08, b: 0.40 alpha: 1}]

    expect(res.length).to.equal(2);
    expect(res[0]).to.equal('col/rgb');

    colourValues = res[1];
    expect(colourValues.r).to.be.closeTo(0.0253, epsilon);
    expect(colourValues.g).to.be.closeTo(0.1226, epsilon);
    expect(colourValues.b).to.be.closeTo(0.0826, epsilon);
    expect(colourValues.alpha).to.be.closeTo(1, epsilon);
  });

  it('should test required functions', () => {

    expect(compile('4')).
      to.equal(4);

    expect(compile('(* 2 4)')).
      to.deep.equal(['*', 2, 4]);

    expect(compile('(- 2 4 5)')).
      to.deep.equal(['-', 2, 4, 5]);

    expect(compile('(+ (/ 2 1) (/ 9 8))')).
      to.deep.equal(['+', ['/', 2, 1], ['/', 9, 8]]);

    expect(compile('(show 2 4)')).
      to.deep.equal(['show', 2, 4]);

    expect(compile('(show [2 4])')).
      to.deep.equal(['show', ['list', 2, 4]]);

    expect(compile('(shot true 4)')).
      to.deep.equal(['shot', '#t', 4]);

    expect(compile('(shoe \'linear)')).
      to.deep.equal(['shoe', ['quote', 'linear']]);

    expect(compile('(slow something 4)')).
      to.deep.equal(['slow', 'something', 4]);

    expect(compile('(how \"something\" 4)')).
      to.deep.equal(['how', ['__string', 'something'], 4]);

    expect(compile('(go arg1: 45 arg2: 11)')).
      to.deep.equal(['go', {arg1: 45, arg2: 11}]);

    expect(compile('(go)')).
      to.deep.equal(['go']);

  });

  it('should compile function define statements', () => {
    expect(compile('(define (add x: 0 y: 0))')).
      to.deep.equal(['define', ['add', {x: 0, y: 0}]]);

    expect(compile('(define (add x: (+ 1 1) y: 0))')).
      to.deep.equal(['define', ['add', {x: ['+', 1, 1], y: 0}]]);
  });
});
