export class Token {

  // constants
  get UNKNOWN()            { return 0; }
  get LIST_START()         { return 1; }
  get LIST_END()           { return 2; }
  get BRACKET_START()      { return 3; }
  get BRACKET_END()        { return 4; }
  get INT()                { return 5; }
  get FLOAT()              { return 6; }
  get NAME()               { return 7; }
  get STRING()             { return 8; }
  get QUOTE_ABBREVIATION() { return 9; }
  
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
