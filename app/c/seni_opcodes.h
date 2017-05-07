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

// Jump the instruction pointer [arg] forward.
OPCODE(JUMP, 0)
// Pop and if not truthy then jump the instruction pointer [arg] forward.
OPCODE(JUMP_IF, -1)


// TODO: make the compiler spot cases where the stack would grow linearly and fix/warn
// would then be able to get rid of MARK/UNMARK
//
// place a mark
OPCODE(MARK, 1)
// keep popping until mark is reached and popped
OPCODE(UNMARK, -1)

OPCODE (NOP, 0)
OPCODE (STOP, 0)
