// label
// goto
// if-goto

// function
// call
// return

OPCODE(PUSH, 1)
OPCODE(POP, -1)

OPCODE(ADD, -1)
OPCODE(SUB, -1)
OPCODE(NEG, 0)

OPCODE(EQ, -1)
OPCODE(GT, -1)
OPCODE(LT, -1)


OPCODE(AND, -1)
OPCODE(OR, -1)
OPCODE(NOT, 0)

OPCODE (NOP, 0)
OPCODE (STOP, 0)
