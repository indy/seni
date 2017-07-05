// load (push) a seni_var onto the stack
OPCODE("LOAD", LOAD, 1)
// store (pop) a seni_var from the stack
OPCODE("STORE", STORE, -1)

// pops the 2 f32 from the top of the stack,
// combines them into one VAR_2D and pushes that onto the stack
OPCODE("SQUISH2", SQUISH2, -1)

OPCODE("ADD", ADD, -1)
OPCODE("SUB", SUB, -1)
OPCODE("MUL", MUL, -1)
OPCODE("DIV", DIV, -1)
OPCODE("NEG", NEG, 0)

OPCODE("EQ", EQ, -1)
OPCODE("GT", GT, -1)
OPCODE("LT", LT, -1)

OPCODE("AND", AND, -1)
OPCODE("OR", OR, -1)
OPCODE("NOT", NOT, 0)

// Jump the instruction pointer [arg] forward.
OPCODE("JUMP", JUMP, 0)
// Pop and if not truthy then jump the instruction pointer [arg] forward.
OPCODE("JUMP_IF", JUMP_IF, -1)

// _0 == keep the existing frame, don't push/pop one
//
// reads the function offset and num args from the stack
OPCODE("CALL", CALL, -2)
// reads the function's body offset from the stack
OPCODE("CALL_0", CALL_0, -1)
// RET will push the top value of the last frame onto the current frame
OPCODE("RET", RET, 1)
OPCODE("RET_0", RET_0, 0)

// like CALL and CALL_0 except it reads an index from the stack
// then it indexes into program->fn_info[index]
OPCODE("CALL_F", CALL_F, -1)
OPCODE("CALL_F_0", CALL_F_0, -1)


// calls a native function, leaving the result on the stack
// offset is 0 as the vm->opcode_offset is modified by the native helper function
OPCODE("NATIVE", NATIVE, 0)

// appends item at top to vector at top-1, leaves vector on stack
OPCODE("APPEND", APPEND, -1)
// given a vector on the stack this unpacks it's contents onto the stack
// offset is 0 as the vm->opcode_offset depends on the size of the stack
// can only be used if each element on the lhs is a NODE_NAME
// the first arg of the bytecode is the expected length of the vector
// vm->opcode_offset is modified by the compiler
OPCODE("PILE", PILE, 0)

// decrements ref count of seni_var at given memory location
OPCODE("DEC_RC", DEC_RC, 0)
OPCODE("INC_RC", INC_RC, 0)

// function look-up versions of DEC_RC, INC_RC, STORE
// they will pop a value from the stack which is the index into program->fn_info
// this will then be used along with the argument's iname to find the index into the MEM_SEG_ARGUMENT memory
OPCODE("FLU_DEC_RC", FLU_DEC_RC, -1)
OPCODE("FLU_INC_RC", FLU_INC_RC, -1)
OPCODE("FLU_STORE", FLU_STORE, -2)

// temporary opcodes which are replaced by their non-placeholder versions during a compilation pass
OPCODE("PLACEHOLDER_DEC_RC", PLACEHOLDER_DEC_RC, 0)
OPCODE("PLACEHOLDER_INC_RC", PLACEHOLDER_INC_RC, 0)
OPCODE("PLACEHOLDER_STORE", PLACEHOLDER_STORE, -1)


// push/pop matrix stack
OPCODE("MTX_LOAD", MTX_LOAD, 0)
OPCODE("MTX_STORE", MTX_STORE, 0)

OPCODE("NOP", NOP, 0)
OPCODE("STOP", STOP, 0)


