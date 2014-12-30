export var NodeType = {
  LIST: 0,
  INT: 1,
  FLOAT: 2,
  NAME: 3,
  LABEL: 4,
  STRING: 5,
  BOOLEAN: 6,
  LAMBDA: 7,
  SPECIAL: 8,
  COLOUR: 9,
  NULL: 10,
};

export class Node {

  constructor(type, value, alterable) {
    this.type = type;
    this.value = value;
    this.alterable = alterable;
    this.children = [];
    
    // node mutate specific
    this.parameterAST = [];
    this.genSym = "";
  }

  getType() {
    return this.type;
  }

  getValue() {
    return this.value;
  }

  isAlterable() {
    return this.alterable;
  }

  addChild(child) {
    this.children.push(child);
  }

  getChild(nth) {
    return this.children[nth];
  }

  getChildren() {
    return this.children;
  }

  size() {
    return this.children.length;
  }

  getGenSym() {
    return this.getGenSym;
  }

  setGenSym(genSym) {
    this.getGenSym = genSym;
  }

  addParameterNode(parameter) {
    this.parameterAST.push(parameter);
  }

  getParameterNodes() {
    return this.parameterAST;
  }
}


