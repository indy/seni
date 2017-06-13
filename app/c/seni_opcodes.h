
OPCODE(PUSH, 1)
OPCODE(POP, -1)

OPCODE(ADD, -1)
OPCODE(SUB, -1)
OPCODE(MUL, -1)
OPCODE(DIV, -1)
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

// _0 == keep the existing frame, don't push/pop one
// CALL_0 is 1 because it results in a RET call and that will
// push the top value of the last frame onto the current frame
//
OPCODE(CALL, 0)
OPCODE(CALL_0, 1)
OPCODE(RET, 0)
OPCODE(RET_0, 0)

// calls a native function, leaving the result on the stack
// offset is 0 as the vm->opcode_offset is modified by the native helper function
OPCODE(NATIVE, 0)

// appends item at top to vector at top-1, leaves vector on stack
OPCODE(APPEND, -1)
// top item is nth, top-1 is vec, pops top and pushes vec[nth]
OPCODE(NTH, 0)
// given a vector on the stack this unpacks it's contents onto the stack
// offset is 0 as the vm->opcode_offset depends on the size of the stack
// can only be used if each element on the lhs is a NODE_NAME
// the first arg of the bytecode is the expected length of the vector
OPCODE(PILE, 0)

// decrements ref count of seni_var at given memory location
OPCODE(DEC_RC, 0)
OPCODE(INC_RC, 0)

// push/pop matrix stack
OPCODE(MTX_PUSH, 0)
OPCODE(MTX_POP, 0)

OPCODE (NOP, 0)
OPCODE (STOP, 0)
