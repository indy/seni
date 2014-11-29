export var TokenType = {
  UNKNOWN: 0,
  LIST_START: 1,
  LIST_END: 2,
  BRACKET_START: 3,
  BRACKET_END: 4,
  INT: 5,
  FLOAT: 6,
  NAME: 7,
  STRING: 8,
  QUOTE_ABBREVIATION: 9
};

export class Token {
  
  constructor(type, value = undefined) {
    this.type = type;
    this.value = value;
  }

  getValue() {
    return this.value;
  }

  getType() {
    return this.type;
  }
}
