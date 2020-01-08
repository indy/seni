// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/*!
The public API for the Seni language

- [Keywords](#keywords)
- [Misc](#misc)
- [Shapes](#shapes): rendering functions
- [Transforms](#transforms): matrix operations
- [Colour](#colour)
- [Math](#math)
- [Prng](#prng): pseudo-random number generation
- [Interp](#interp): parametrically define value
- [Path](#path): invoke a function for every point
- [Repeat](#repeat): invoke a function repeatedly, altering the base view matrix
- [Focal](#focal): answer the question: how important is this co-ordinate?
- [Bitmap](#bitmap): working with bitmaps
- [Masking](#masking): masking parts of the canvas
- [Gen](#gen): define how genotypes will be generated
- [Interpolation Constants](#interpolation-constants)

# Keywords

- each : iterate through each item in a vector
- loop : loop through a series of values
- fence : loop through a series of values
- address-of : address of a function
- fn-call : invokes the function at the given address
- define : define a binding
- fn : define a function
- [++](#++) : appends to a vector

# Misc

- [image](#image) : colour manipulation for the final image
- [debug/print](#debugprint) : prints debug
- [nth](#nth)
- [vector/length](#vectorlength)
- [probe](#probe) : writes to the vm's debug_str (used for testing only)
- [get-x](#getx)
- [get-y](#gety)


# Shapes

 - [line](#line)
 - [rect](#rect) : draws rect centered at position
 - [circle](#circle)
 - [circle-slice](#circle-slice)
 - [ring](#ring)
 - [poly](#poly)
 - [quadratic](#quadratic)
 - [bezier](#bezier)
 - [bezier-bulging](#bezier-bulging)
 - [stroked-bezier](#stroked-bezier)

# Transforms

- [translate](#translate)
- [rotate](#rotate)
- [scale](#scale)

# Colour

- [col/convert](#colconvert)
- [col/rgb](#colrgb)
- [col/hsl](#colhsl)
- [col/hsluv](#colhsluv)
- [col/hsv](#colhsv)
- [col/lab](#collab)
- [col/complementary](#colcomplementary)
- [col/split-complementary](#colsplit-complementary)
- [col/analagous](#colanalagous)
- [col/triad](#coltriad)
- [col/darken](#coldarken)
- [col/lighten](#collighten)
- [col/e0](#cole0)
- [col/e1](#cole1)
- [col/e2](#cole2)
- [col/alpha](#colealpha)
- [col/set-e0](#colset-e0)
- [col/set-e1](#colset-e1)
- [col/set-e2](#colset-e2)
- [col/set-alpha](#colset-alpha)
- [col/add-e0](#coladd-e0)
- [col/add-e1](#coladd-e1)
- [col/add-e2](#coladd-e2)
- [col/add-alpha](#coladd-alpha)
- [col/build-procedural](#colbuild-procedural)
- [col/build-bezier](#colbuild-bezier)
- [col/value](#colvalue)

# Math

- [math/distance](#mathdistance)
- [math/normal](#mathnormal)
- [math/clamp](#mathclamp)
- [math/radians->degrees](#mathradians->degrees)
- [math/cos](#mathcos)
- [math/sin](#mathsin)

# Prng

- [prng/build](#prngbuild)
- [prng/values](#prngvalues)
- [prng/value](#prngvalue)
- [prng/perlin](#prngperlin)

# Interp

- [interp/build](#interpbuild)
- [interp/value](#interpvalue)
- [interp/cos](#interpcos)
- [interp/sin](#interpsin)
- [interp/bezier](#interpbezier)
- [interp/bezier-tangent](#interpbezier-tangent)
- [interp/ray](#interpray)
- [interp/line](#interpline)
- [interp/circle](#interpcircle)

# Path

- [path/linear](#pathlinear)
- [path/circle](#pathcircle)
- [path/spline](#pathspline)
- [path/bezier](#pathbezier)

# Repeat

- [repeat/symmetry-vertical](#repeatsymmetry-vertical)
- [repeat/symmetry-horizontal](#repeatsymmetry-horizontal)
- [repeat/symmetry-4](#repeatsymmetry-4)
- [repeat/symmetry-8](#repeatsymmetry-8)
- [repeat/rotate](#repeatrotate)
- [repeat/rotate-mirrored](#repeatrotate-mirrored)

# Focal

- [focal/build-point](#focalbuild-point)
- [focal/build-vline](#focalbuild-vline)
- [focal/build-hline](#focalbuild-hline)
- [focal/value](#focalvalue)

# Bitmap

- [bitmap/each](#bitmapeach)
- [bitmap/value](#bitmapvalue)
- [bitmap/width](#bitmapwidth)
- [bitmap/height](#bitmapheight)

# Masking

- [mask/set](#maskset)

# Gen

- [gen/stray-int](#genstray-int)
- [gen/stray](#genstray)
- [gen/stray-2d](#genstray-2d)
- [gen/stray-3d](#genstray-3d)
- [gen/stray-4d](#genstray-4d)
- [gen/int](#genint)
- [gen/scalar](#genscalar)
- [gen/2d](#gen2d)
- [gen/select](#genselect)
- [gen/col](#gencol)

# Misc functions

## image

Parameter | Default | Description
--- | --- | ---
contrast | 1 |
brightness | 0 |
saturation | 1 |
linear-colour-space | 0 | set to 1 for legacy sketches

## debug/print

Parameter | Default | Description
--- | --- | ---
value | NULL | the value to print

## nth

Parameter | Default | Description
--- | --- | ---
from | NULL | vector to index
n | 0 | 0 based index

## vector/length

Parameter | Default | Description
--- | --- | ---
from | NULL |

## probe

Parameter | Default | Description
--- | --- | ---
scalar | NULL | appends value onto vm.debug_str
vector | NULL | appends v2d onto vm.debug_str
worldspace | NULL | appends v2d in worldspace onto vm.debug_str

## get-x

Parameter | Default | Description
--- | --- | ---
from | NULL |

## get-y

Parameter | Default | Description
--- | --- | ---
from | NULL |

# Shape functions

## line

Parameter | Default | Description
--- | --- | ---
from          | [10 10]      |
to            | [900 500]    |
width         | 4            |
colour        | RGB(0 0 0 1) |
from-colour   | RGB(0 0 0 1) |
to-colour     | RGB(0 0 0 1) |
brush         | brush/flat   |
brush-subtype | 0            |

either set 'colour' or both 'from-colour' and 'to-colour'

## rect

Parameter | Default | Description
--- | --- | ---
| width     | 4            |             |
| height    | 10           |             |
| position  | [10 23]      |             |
| colour    | RGB(0 0 0 1) |             |

## circle

Parameter | Default | Description
--- | --- | ---
| width        |            4 |             |
| height       |           10 |             |
| position     |      [10 23] |             |
| colour       | RGB(0 0 0 1) |             |
| tessellation |           10 |             |
| radius       |           -1 |             |

## circle-slice

Parameter | Default | Description
--- | --- | ---
| width        |            4 |             |
| height       |           10 |             |
| radius       |           -1 |             |
| position     |      [10 23] |             |
| colour       | RGB(0 0 0 1) |             |
| tessellation |           10 |             |
| angle-start  |            0 |             |
| angle-end    |            0 |             |
| inner-width  |            1 |             |
| inner-height |            1 |             |

## ring

Parameter | Default | Description
--- | --- | ---
| inner-radius        |            200 |             |
| outer-radius       |           300 |             |
| position     |      [10 10] |             |
| inner-colour       | RGB(0 0 0 1) |             |
| outer-colour       | RGB(0 0 0 1) |             |
| tessellation |           10 |             |

## poly

Parameter | Default | Description
--- | --- | ---
| coords    | NULL    |             |
| colours   | NULL    |             |

## quadratic

Parameter | Default | Description
--- | --- | ---
| line-width         |            4 |             |
| line-width-start   |            4 |             |
| line-width-end     |            4 |             |
| line-width-mapping |       linear |             |
| coords             |              |             |
| t-start            |            0 |             |
| t-end              |            1 |             |
| tessellation       |           10 |             |
| colour             | RGB(0 0 0 1) |             |
| brush              |   brush/flat |             |
| brush-subtype      |            0 |             |

## bezier

Parameter | Default | Description
--- | --- | ---
| line-width         |            4 |             |
| line-width-start   |            4 |             |
| line-width-end     |            4 |             |
| line-width-mapping |       linear |             |
| coords             |              |             |
| t-start            |            0 |             |
| t-end              |            1 |             |
| tessellation       |           10 |             |
| colour             | RGB(0 0 0 1) |             |
| brush              |   brush/flat |             |
| brush-subtype      |            0 |             |

## bezier-bulging

Parameter | Default | Description
--- | --- | ---
| line-width    |            4 |             |
| coords        |              |             |
| t-start       |            0 |             |
| t-end         |            1 |             |
| tessellation  |           10 |             |
| colour        | RGB(0 0 0 1) |             |
| brush         |   brush/flat |             |
| brush-subtype |            0 |             |

## stroked-bezier

Parameter | Default | Description
--- | --- | ---
| tessellation            |           10 |             |
| coords                  |              |             |
| stroke-tessellation     |           10 |             |
| stroke-noise            |           25 |             |
| stroke-line-width-start |            1 |             |
| stroke-line-width-end   |            1 |             |
| colour                  | RGB(0 0 0 1) |             |
| colour-volatility       |            0 |             |
| seed                    |            0 |             |
| line-width-mapping      |       linear |             |
| brush                   |   brush/flat |             |
| brush-subtype           |            0 |             |

# Transform functions

## translate

Parameter | Default | Description
--- | --- | ---
    | vector    | [0 0]   |             |

## rotate

Parameter | Default | Description
--- | --- | ---
    | angle     |       0 |             |

## scale

Parameter | Default | Description
--- | --- | ---
    | vector    | [1 1]   |             |
    | scale     | 1       |             |

# Colour Functions

## col/convert

Parameter | Default | Description
--- | --- | ---
    | format    | RGB          |             |
    | from      | RGB(0 0 0 1) |             |

## col/rgb

Parameter | Default | Description
--- | --- | ---
    | r         |       0 |        0..1 |
    | g         |       0 |        0..1 |
    | b         |       0 |        0..1 |
    | alpha     |       1 |        0..1 |

## col/hsl

Parameter | Default | Description
--- | --- | ---
    | h         |       0 |      0..360 |
    | s         |       0 |        0..1 |
    | l         |       0 |        0..1 |
    | alpha     |       1 |        0..1 |

## col/hsluv

Parameter | Default | Description
--- | --- | ---
    | h         |       0 |      0..360 |
    | s         |       0 |      0..100 |
    | l         |       0 |      0..100 |
    | alpha     |       1 |        0..1 |

## col/hsv

Parameter | Default | Description
--- | --- | ---
    | h         |       0 |      0..360 |
    | s         |       0 |        0..1 |
    | v         |       0 |        0..1 |
    | alpha     |       1 |        0..1 |


## col/lab

Parameter | Default | Description
--- | --- | ---
    | l         |       0 |         0.. |
    | a         |       0 |       -1..1 |
    | b         |       0 |       -1..1 |
    | alpha     |       1 |        0..1 |

## col/complementary

returns the complimentary colour

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/split-complementary

returns a vector of 2 colours

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/analagous

returns a vector of 2 colours

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/triad

returns a vector of 2 colours

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/darken

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |
    | value     | 0            |      0..100 |

## col/lighten

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |
    | value     | 0            |      0..100 |

## col/e0

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/e1

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/e2

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/alpha

Parameter | Default | Description
--- | --- | ---
    | from    | RGB(0 0 0 1) |             |

## col/set-e0

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/set-e1

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/set-e2

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/set-alpha

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/add-e0

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/add-e1

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/add-e2

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/add-alpha

Parameter | Default | Description
--- | --- | ---
    | from      | RGB(0 0 0 1) |             |
    | value     | 0            |             |

## col/build-procedural

returns COLOUR_FN_PROCEDURAL

Parameter | Default | Description
--- | --- | ---
    | preset    | robocop |             |
    | alpha     | 1       |             |
    | a         | [0 0 0] |             |
    | b         | [0 0 0] |             |
    | c         | [0 0 0] |             |
    | d         | [0 0 0] |             |

## col/build-bezier

returns COLOUR_FN_BEZIER

Parameter | Default | Description
--- | --- | ---
    | a         | RGB(0 0 0 1) |             |
    | b         | RGB(0 0 0 1) |             |
    | c         | RGB(0 0 0 1) |             |
    | d         | RGB(0 0 0 1) |             |

## col/value

Parameter | Default | Description
--- | --- | ---
    | from      | NULL    | either a FN_PROCEDURAL or FN_BEZIER |
    | t         | 0       |                                     |

# Math Functions

## math/distance

Parameter | Default | Description
--- | --- | ---
    | vec1      | [0 0]   |             |
    | vec2      | [0 0]   |             |

## math/normal

Parameter | Default | Description
--- | --- | ---
    | vec1      | [0 0]   |             |
    | vec2      | [0 0]   |             |

## math/clamp

Parameter | Default | Description
--- | --- | ---
    | from      |       0 |             |
    | min       |       0 |             |
    | max       |       1 |             |

## math/radians->degrees

Parameter | Default | Description
--- | --- | ---
    | from     | 0       |             |

## math/cos

Parameter | Default | Description
--- | --- | ---
    | from     | 0       |             |

## math/sin

Parameter | Default | Description
--- | --- | ---
    | from     | 0       |             |

# Prng Functions

## prng/build

Parameter | Default | Description
--- | --- | ---
    | seed      |   12322 |             |
    | min       |       0 |             |
    | max       |       1 |             |

## prng/values

Parameter | Default | Description
--- | --- | ---
    | from      |         |             |
    | num       |         |             |

## prng/value

Parameter | Default | Description
--- | --- | ---
    | from      |         |             |

## prng/perlin

Parameter | Default | Description
--- | --- | ---
    | x         |       1 |             |
    | y         |       1 |             |
    | z         |       1 |             |

# Interp Functions

## interp/build

Parameter | Default | Description
--- | --- | ---
    | from      | [0 1]   |             |
    | to        | [0 100] |             |
    | clamping  | false   |             |
    | mapping   | linear  |             |

## interp/value

Parameter | Default | Description
--- | --- | ---
    | from      |         |             |
    | t         | 0       |             |

## interp/cos

Parameter | Default | Description
--- | --- | ---
    | amplitude |       1 |             |
    | frequency |       1 |             |
    | t         |       1 |             |

## interp/sin

Parameter | Default | Description
--- | --- | ---
    | amplitude |       1 |             |
    | frequency |       1 |             |
    | t         |       1 |             |

## interp/bezier

Parameter | Default | Description
--- | --- | ---
    | coords    |         |             |
    | t         |         |             |

## interp/bezier-tangent

Parameter | Default | Description
--- | --- | ---
    | coords    |         |             |
    | t         |         |             |

## interp/ray

Parameter | Default | Description
--- | --- | ---
    | point     | [0 0]       |             |
    | direction | [1000 1000] |             |
    | t         | 0           |             |

## interp/line

Parameter | Default | Description
--- | --- | ---
    | from      | [0 0]       |             |
    | to        | [1000 1000] |             |
    | clamping  | false       |             |
    | mapping   | linear      |             |
    | t         | 0           |             |

## interp/circle

Parameter | Default | Description
--- | --- | ---
    | position  |   [0 0] |             |
    | radius    |       1 |             |
    | t         |       0 |             |

# Path Functions

## path/linear

Parameter | Default | Description
--- | --- | ---
    | from      |     [0 0] |             |
    | to        | [100 100] |             |
    | steps     |        10 |             |
    | t-start   |         0 |             |
    | t-end     |         1 |             |
    | fn        |           |             |
    | mapping   | linear    |             |

## path/circle

Parameter | Default | Description
--- | --- | ---
    | position  |   [0 0] |             |
    | radius    |     100 |             |
    | steps     |      10 |             |
    | t-start   |       0 |             |
    | t-end     |       1 |             |
    | fn        |         |             |
    | mapping   |  linear |             |

## path/spline

Parameter | Default | Description
--- | --- | ---
    | coords    |         |             |
    | steps     |      10 |             |
    | t-start   |       0 |             |
    | t-end     |       1 |             |
    | fn        |         |             |
    | mapping   |  linear |             |

## path/bezier

Parameter | Default | Description
--- | --- | ---
    | coords    |         |             |
    | steps     |      10 |             |
    | t-start   |       0 |             |
    | t-end     |       1 |             |
    | fn        |         |             |
    | mapping   |  linear |             |

# Repeat Functions

Parameter | Default | Description
--- | --- | ---


Parameter | Default | Description
--- | --- | ---
    | fn        |         |             |

## repeat/symmetry-horizontal

Parameter | Default | Description
--- | --- | ---
    | fn        |         |             |

## repeat/symmetry-4

Parameter | Default | Description
--- | --- | ---
    | fn        |         |             |

## repeat/symmetry-8

Parameter | Default | Description
--- | --- | ---
    | fn        |         |             |

## repeat/rotate

Parameter | Default | Description
--- | --- | ---
    | fn        |         |             |
    | copies    | 3       |             |

## repeat/rotate-mirrored

Parameter | Default | Description
--- | --- | ---
    | fn        |         |             |
    | copies    |       3 |             |

# Focal Functions

## focal/build-point

Parameter | Default | Description
--- | --- | ---
    | position           | [0 0]   |                                         |
    | distance           | 1       |                                         |
    | mapping            | linear  | [interpolation constant](#interpolation-constants)                  |
    | transform-position | 1       | 0 to not apply current transform matrix |


## focal/build-vline

Parameter | Default | Description
--- | --- | ---
    | position           | [0 0]   |                                         |
    | distance           | 1       |                                         |
    | mapping            | linear  | [interpolation constant](#interpolation-constants)                  |
    | transform-position | 1       | 0 to not apply current transform matrix |

## focal/build-hline

Parameter | Default | Description
--- | --- | ---
    | position           | [0 0]   |                                         |
    | distance           | 1       |                                         |
    | mapping            | linear  | [interpolation constant](#interpolation-constants)                  |
    | transform-position | 1       | 0 to not apply current transform matrix |

## focal/value

Parameter | Default | Description
--- | --- | ---
    | from      |         |             |
    | position  | [0 0]   |             |


# Bitmap Functions

## bitmap/each

Parameter | Default | Description
--- | --- | ---
    | from          |               |             |
    | position      | [canvas/width / 2, canvas/height / 2] |             |
    | width         | canvas/width  |             |
    | height        | canvas/height |             |
    | fn            |               |             |
    | shuffle-seed  | 0.0           |             |

## bitmap/value

Parameter | Default | Description
--- | --- | ---
    | from           |              |             |
    | position       | [0 0]        |             |
    | default-colour | RGB(0 0 0 0) |             |

## bitmap/width

Parameter | Default | Description
--- | --- | ---
    | from           |                  |             |

## bitmap/height

Parameter | Default | Description
--- | --- | ---
    | from           |                  |             |

# Masking Functions

## mask/set

Parameter | Default | Description
--- | --- | ---
    | TEMP      |       0 |             |
    | TEMP        |       1 |             |

# Gen Functions

## gen/stray-int

Parameter | Default | Description
--- | --- | ---
    | from      |       0 |             |
    | by        |       1 |             |

## gen/stray

Parameter | Default | Description
--- | --- | ---
    | from      |       1 |             |
    | by        |     0.2 |             |

## gen/stray-2d

Parameter | Default | Description
--- | --- | ---
    | from      | [10 10] |             |
    | by        | [1 1]   |             |

## gen/stray-3d

Parameter | Default | Description
--- | --- | ---
    | from      | [10 10 10] |             |
    | by        | [1 1 1]    |             |

## gen/stray-4d

Parameter | Default | Description
--- | --- | ---
    | from      | [10 10 10 10] |             |
    | by        | [1 1 1 1]     |             |

## gen/int

Parameter | Default | Description
--- | --- | ---
    | min       |       0 |             |
    | max       |    1000 |             |

## gen/scalar

Parameter | Default | Description
--- | --- | ---
    | min       |       0 |             |
    | max       |       1 |             |

## gen/2d

Parameter | Default | Description
--- | --- | ---
    | min       |       0 |             |
    | max       |       1 |             |

## gen/select

Parameter | Default | Description
--- | --- | ---
    | from      | NULL    |             |

## gen/col

Parameter | Default | Description
--- | --- | ---
    | alpha     |         |             |


# Interpolation Constants

Name | Description
--- | ---
   | linear                  |             |
   | ease/quick              |             |
   | ease/slow-in            |             |
   | ease/slow-in-out        |             |
   | ease/quadratic-in       |             |
   | ease/quadratic-out      |             |
   | ease/quadratic-in-out   |             |
   | ease/cubic-in           |             |
   | ease/cubic-out          |             |
   | ease/cubic-in-out       |             |
   | ease/quartic-in         |             |
   | ease/quartic-out        |             |
   | ease/quartic-in-out     |             |
   | ease/quintic-in         |             |
   | ease/quintic-out        |             |
   | ease/quintic-in-out     |             |
   | ease/sin-in             |             |
   | ease/sin-out            |             |
   | ease/sin-in-out         |             |
   | ease/circular-in        |             |
   | ease/circular-out       |             |
   | ease/circular-in-out    |             |
   | ease/exponential-in     |             |
   | ease/exponential-out    |             |
   | ease/exponential-in-out |             |
   | ease/elastic-in         |             |
   | ease/elastic-out        |             |
   | ease/elastic-in-out     |             |
   | ease/back-in            |             |
   | ease/back-out           |             |
   | ease/back-in-out        |             |
   | ease/bounce-in          |             |
   | ease/bounce-out         |             |
   | ease/bounce-in-out      |             |


*/
