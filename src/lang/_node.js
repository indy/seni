class Node {
  constructor(type, value, alterable) {
    this.type = type;
    this.value = value;
    this.alterable = alterable;
    this.children = [];

    // node mutate specific
    this.parameterAST = [];
    this.genSym = '';
  }

  addChild(child) {
    this.children.push(child);
  }

  getChild(nth) {
    return this.children[nth];
  }

  size() {
    return this.children.length;
  }

  addParameterNode(parameter) {
    this.parameterAST.push(parameter);
  }
}

export default Node;
