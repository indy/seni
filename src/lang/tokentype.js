const TokenType = {
  UNKNOWN: 0,
  LIST_START: 1,
  LIST_END: 2,
  BRACKET_START: 3,
  BRACKET_END: 4,
  INT: 5,
  FLOAT: 6,
  NAME: 7,
  STRING: 8,
  QUOTE_ABBREVIATION: 9,
  LABEL: 10,
  COMMENT: 11
};

export default TokenType;
