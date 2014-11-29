export var NodeType = {
  LIST: 0,
  INT: 1,
  FLOAT: 2,
  NAME: 3,
  STRING: 4,
  BOOLEAN: 5,
  LAMBDA: 6,
  SPECIAL: 7,
  COLOUR: 8,
  NULL: 9
};

export class Node {


  getType() {
    return this.type;
  }

  // todo: should this go in NodeList?
  addChild(child) {
  }
}
