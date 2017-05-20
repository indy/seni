
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

OPCODE(CALL, 0)
// KF == KEEP FRAME
// CALL_KF is 1 because it results in a RET call and that will
// push the top value of the last frame onto the current frame
//
OPCODE(CALL_KF, 1)
OPCODE(RET, 0)
OPCODE(RET_KF, 0)

OPCODE (NOP, 0)
OPCODE (STOP, 0)
