REGISTER_KEYWORD("false", INAME_FALSE)
REGISTER_KEYWORD("true", INAME_TRUE)

// mathematical special forms
//
REGISTER_KEYWORD("+", INAME_PLUS)
REGISTER_KEYWORD("-", INAME_MINUS)
REGISTER_KEYWORD("*", INAME_MULT)
REGISTER_KEYWORD("/", INAME_DIVIDE)
REGISTER_KEYWORD("=", INAME_EQUAL)
REGISTER_KEYWORD(">", INAME_GT)
REGISTER_KEYWORD("<", INAME_LT)

// built-in keywords/special-forms
//
REGISTER_KEYWORD("vector/append", INAME_VECTOR_APPEND)
REGISTER_KEYWORD("sqrt", INAME_SQRT)
REGISTER_KEYWORD("mod", INAME_MOD)
REGISTER_KEYWORD("and", INAME_AND)
REGISTER_KEYWORD("or", INAME_OR)
REGISTER_KEYWORD("not", INAME_NOT)
REGISTER_KEYWORD("define", INAME_DEFINE)
REGISTER_KEYWORD("fn", INAME_FN)
REGISTER_KEYWORD("if", INAME_IF)
REGISTER_KEYWORD("loop", INAME_LOOP)
REGISTER_KEYWORD("on-matrix-stack", INAME_ON_MATRIX_STACK)
REGISTER_KEYWORD("setq", INAME_SETQ)
REGISTER_KEYWORD("address-of", INAME_ADDRESS_OF)
REGISTER_KEYWORD("fn-call", INAME_FN_CALL)

// pre-defined globals
//
REGISTER_KEYWORD("#vars", INAME_HASH_VARS)
REGISTER_KEYWORD("canvas/width", INAME_CANVAS_WIDTH)
REGISTER_KEYWORD("canvas/height", INAME_CANVAS_HEIGHT)
REGISTER_KEYWORD("math/TAU", INAME_MATH_TAU)

// enumerations for colours and colour formats
//
REGISTER_KEYWORD("RGB", INAME_RGB)
REGISTER_KEYWORD("HSL", INAME_HSL)
REGISTER_KEYWORD("LAB", INAME_LAB)
REGISTER_KEYWORD("HSV", INAME_HSV)
REGISTER_KEYWORD("white", INAME_WHITE)
REGISTER_KEYWORD("black", INAME_BLACK)
REGISTER_KEYWORD("red", INAME_RED)
REGISTER_KEYWORD("green", INAME_GREEN)
REGISTER_KEYWORD("blue", INAME_BLUE)
REGISTER_KEYWORD("yellow", INAME_YELLOW)
REGISTER_KEYWORD("magenta", INAME_MAGENTA)
REGISTER_KEYWORD("cyan", INAME_CYAN)

// enumerations for procedural colours
//
REGISTER_KEYWORD("chrome", INAME_CHROME)
REGISTER_KEYWORD("hotline-miami", INAME_HOTLINE_MIAMI)
REGISTER_KEYWORD("knight-rider", INAME_KNIGHT_RIDER)
REGISTER_KEYWORD("mars", INAME_MARS)
REGISTER_KEYWORD("rainbow", INAME_RAINBOW)
REGISTER_KEYWORD("robocop", INAME_ROBOCOP)
REGISTER_KEYWORD("transformers", INAME_TRANSFORMERS)

// enumerations for brush types
//
REGISTER_KEYWORD("brush-flat", INAME_BRUSH_FLAT)
REGISTER_KEYWORD("brush-a", INAME_BRUSH_A)
REGISTER_KEYWORD("brush-b", INAME_BRUSH_B)
REGISTER_KEYWORD("brush-c", INAME_BRUSH_C)
REGISTER_KEYWORD("brush-d", INAME_BRUSH_D)
REGISTER_KEYWORD("brush-e", INAME_BRUSH_E)
REGISTER_KEYWORD("brush-f", INAME_BRUSH_F)
REGISTER_KEYWORD("brush-g", INAME_BRUSH_G)

// enumerations for interpolation
//
REGISTER_KEYWORD("linear", INAME_LINEAR)
REGISTER_KEYWORD("quick", INAME_QUICK)
REGISTER_KEYWORD("slow-in", INAME_SLOW_IN)
REGISTER_KEYWORD("slow-in-out", INAME_SLOW_IN_OUT)

// common parameter labels
//
REGISTER_KEYWORD("a", INAME_A)
REGISTER_KEYWORD("b", INAME_B)
REGISTER_KEYWORD("c", INAME_C)
REGISTER_KEYWORD("d", INAME_D)
REGISTER_KEYWORD("g", INAME_G)
REGISTER_KEYWORD("h", INAME_H)
REGISTER_KEYWORD("l", INAME_L)
REGISTER_KEYWORD("n", INAME_N)
REGISTER_KEYWORD("r", INAME_R)
REGISTER_KEYWORD("s", INAME_S)
REGISTER_KEYWORD("t", INAME_T)
REGISTER_KEYWORD("v", INAME_V)
REGISTER_KEYWORD("x", INAME_X)
REGISTER_KEYWORD("y", INAME_Y)
REGISTER_KEYWORD("z", INAME_Z)
REGISTER_KEYWORD("alpha", INAME_ALPHA)
REGISTER_KEYWORD("amplitude", INAME_AMPLITUDE)
REGISTER_KEYWORD("angle", INAME_ANGLE)
REGISTER_KEYWORD("brush", INAME_BRUSH)
REGISTER_KEYWORD("brush-subtype", INAME_BRUSH_SUBTYPE)
REGISTER_KEYWORD("clamping", INAME_CLAMPING)
REGISTER_KEYWORD("colour", INAME_COLOUR)
REGISTER_KEYWORD("colour-volatility", INAME_COLOUR_VOLATILITY)
REGISTER_KEYWORD("colours", INAME_COLOURS)
REGISTER_KEYWORD("coords", INAME_COORDS)
REGISTER_KEYWORD("copies", INAME_COPIES)
REGISTER_KEYWORD("distance", INAME_DISTANCE)
REGISTER_KEYWORD("format", INAME_FORMAT)
REGISTER_KEYWORD("frequency", INAME_FREQUENCY)
REGISTER_KEYWORD("from", INAME_FROM)
REGISTER_KEYWORD("height", INAME_HEIGHT)
REGISTER_KEYWORD("increment", INAME_INCREMENT)
REGISTER_KEYWORD("iterations", INAME_ITERATIONS)
REGISTER_KEYWORD("line-width", INAME_LINE_WIDTH)
REGISTER_KEYWORD("line-width-end", INAME_LINE_WIDTH_END)
REGISTER_KEYWORD("line-width-mapping", INAME_LINE_WIDTH_MAPPING)
REGISTER_KEYWORD("line-width-start", INAME_LINE_WIDTH_START)
REGISTER_KEYWORD("mapping", INAME_MAPPING)
REGISTER_KEYWORD("max", INAME_MAX)
REGISTER_KEYWORD("min", INAME_MIN)
REGISTER_KEYWORD("num", INAME_NUM)
REGISTER_KEYWORD("overlap", INAME_OVERLAP)
REGISTER_KEYWORD("position", INAME_POSITION)
REGISTER_KEYWORD("preset", INAME_PRESET)
REGISTER_KEYWORD("radius", INAME_RADIUS)
REGISTER_KEYWORD("scalar", INAME_SCALAR)
REGISTER_KEYWORD("seed", INAME_SEED)
REGISTER_KEYWORD("step", INAME_STEP)
REGISTER_KEYWORD("steps", INAME_STEPS)
REGISTER_KEYWORD("stroke-line-width-end", INAME_STROKE_LINE_WIDTH_END)
REGISTER_KEYWORD("stroke-line-width-start", INAME_STROKE_LINE_WIDTH_START)
REGISTER_KEYWORD("stroke-noise", INAME_STROKE_NOISE)
REGISTER_KEYWORD("stroke-tessellation", INAME_STROKE_TESSELLATION)
REGISTER_KEYWORD("t-end", INAME_T_END)
REGISTER_KEYWORD("t-start", INAME_T_START)
REGISTER_KEYWORD("tessellation", INAME_TESSELLATION)
REGISTER_KEYWORD("to", INAME_TO)
REGISTER_KEYWORD("upto", INAME_UPTO)
REGISTER_KEYWORD("value", INAME_VALUE)
REGISTER_KEYWORD("vec1", INAME_VEC1)
REGISTER_KEYWORD("vec2", INAME_VEC2)
REGISTER_KEYWORD("vector", INAME_VECTOR)
REGISTER_KEYWORD("volatility", INAME_VOLATILITY)
REGISTER_KEYWORD("width", INAME_WIDTH)
REGISTER_KEYWORD("inner-width", INAME_INNER_WIDTH)
REGISTER_KEYWORD("inner-height", INAME_INNER_HEIGHT)
REGISTER_KEYWORD("angle-start", INAME_ANGLE_START)
REGISTER_KEYWORD("angle-end", INAME_ANGLE_END)
