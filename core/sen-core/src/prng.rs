// Copyright (C) 2019 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::mathutil::{clamp, lerp};
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

const PERMUTATIONS: [usize; 512] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180, 151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194,
    233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234,
    75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174,
    20, 125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83,
    111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25,
    63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188,
    159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147,
    118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
    213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253,
    19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193,
    238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31,
    181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

const BASIS: [[f32; 3]; 12] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
];

// perlin's gradient has 12 cases so some get used 1/16th of the time
// and some 2/16ths. We reduce bias by changing those fractions
// to 5/64ths and 6/64ths, and the same 4 cases get the extra weight.
const INDICES: [usize; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 9, 1, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 1,
    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 1, 2, 3, 4, 5, 6, 7,
    8, 9, 10, 11,
];

#[derive(Clone, Debug)]
pub struct PrngStateStruct {
    rng: XorShiftRng,
    min: f32,
    max: f32,
}

fn as_u8(i: i32) -> u8 {
    (i % (1 << 8)) as u8
}

impl PrngStateStruct {
    pub fn new(seed: i32, min: f32, max: f32) -> Self {
        PrngStateStruct {
            rng: PrngStateStruct::make_xor_shift_rng(seed),
            min,
            max,
        }
    }

    pub fn set_state(&mut self, seed: i32) {
        self.rng = PrngStateStruct::make_xor_shift_rng(seed);
    }

    // pub fn prng_i32_range(&mut self, min: i32, max: i32) -> i32 {
    //     self.rnd_pcg_range(min, max)
    // }

    // 0..1
    pub fn prng_f32(&mut self) -> f32 {
        let a = self.rng.next_u32();

        a as f32 / std::u32::MAX as f32
    }

    pub fn prng_f32_range(&mut self, min: f32, max: f32) -> f32 {
        let value = self.prng_f32();
        (value * (max - min)) + min
    }

    pub fn prng_f32_defined_range(&mut self) -> f32 {
        let value = self.prng_f32();
        (value * (self.max - self.min)) + self.min
    }

    pub fn prng_f32_around(&mut self, val: f32, percent: f32, min: f32, max: f32) -> f32 {
        let value = self.prng_f32();
        let range = ((max - min) / 100.0) * percent;
        let lowest = val - range;
        let highest = val + range;
        let res = (value * (highest - lowest)) + lowest;

        clamp(res, min, max)
    }

    fn make_xor_shift_rng(seed: i32) -> XorShiftRng {
        let seedy: [u8; 16] = [
            as_u8(seed),
            as_u8(seed + 1),
            as_u8(seed + seed - 2),
            as_u8(seed + 3),
            as_u8(seed + seed - 4),
            as_u8(seed + 5),
            as_u8(seed + seed - 6),
            as_u8(seed + 7),
            as_u8(seed + seed - 8),
            as_u8(seed + 9),
            as_u8(seed + seed - 10),
            as_u8(seed + 11),
            as_u8(seed + seed - 12),
            as_u8(seed + 13),
            as_u8(seed + seed - 14),
            as_u8(seed + 15),
        ];

        XorShiftRng::from_seed(seedy)
    }
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn grad(hash: usize, x: f32, y: f32, z: f32) -> f32 {
    let grad = BASIS[INDICES[hash & 15]];
    grad[0] * x + grad[1] * y + grad[2] * z
}

// A basic translation of Ken Perlin's Java
// reference implementation of improved noise (C) 2002

// returns a value in the range -1..1
pub fn perlin(x: f32, y: f32, z: f32) -> f32 {
    let x_floor = x.floor() as i32;
    let y_floor = y.floor() as i32;
    let z_floor = z.floor() as i32;

    let xf = (x_floor & 255) as usize;
    let yf = (y_floor & 255) as usize;
    let zf = (z_floor & 255) as usize;

    let x = x - x_floor as f32;
    let y = y - y_floor as f32;
    let z = z - z_floor as f32;

    let u = fade(x);
    let v = fade(y);
    let w = fade(z);

    let a = PERMUTATIONS[xf] + yf;
    let aa = PERMUTATIONS[a] + zf;
    let ab = PERMUTATIONS[a + 1] + zf;
    let b = PERMUTATIONS[xf + 1] + yf;
    let ba = PERMUTATIONS[b] + zf;
    let bb = PERMUTATIONS[b + 1] + zf;

    return lerp(
        w,
        lerp(
            v,
            lerp(
                u,
                grad(PERMUTATIONS[aa], x, y, z), // AND ADD
                grad(PERMUTATIONS[ba], x - 1.0, y, z),
            ), // BLENDED
            lerp(
                u,
                grad(PERMUTATIONS[ab], x, y - 1.0, z), // RESULTS
                grad(PERMUTATIONS[bb], x - 1.0, y - 1.0, z),
            ),
        ), // FROM  8
        lerp(
            v,
            lerp(
                u,
                grad(PERMUTATIONS[aa + 1], x, y, z - 1.0), // CORNERS
                grad(PERMUTATIONS[ba + 1], x - 1.0, y, z - 1.0),
            ), // OF CUBE
            lerp(
                u,
                grad(PERMUTATIONS[ab + 1], x, y - 1.0, z - 1.0),
                grad(PERMUTATIONS[bb + 1], x - 1.0, y - 1.0, z - 1.0),
            ),
        ),
    );
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::vm::tests::*;

    #[test]
    pub fn test_prng_value() {
        is_debug_str(
            "(define p (prng/build seed: 5 min: 0 max: 1))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))",
            "0.36151162 0.4412291 0.37725854 0.5783009 0.65834785 0.48318255 0.7342905 0.44944018 0.56456256",
        );
    }

    #[test]
    pub fn test_prng_value2() {
        is_debug_str(
            "(define p (prng/build seed: 5938 min: 3 max: 9))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))
             (probe scalar: (prng/value from: p))",
            "5.3797703 7.0697656 4.541839 7.1620464 6.7335386 3.5603561 3.4189374 3.1284251 8.890152",
        );
    }

    #[test]
    pub fn test_prng_values() {
        is_debug_str(
            "(define p (prng/build seed: 5 min: 0 max: 1))
             (define vs (prng/values from: p num: 3))
             (probe scalar: (nth from: vs n: 0))
             (probe scalar: (nth from: vs n: 1))
             (probe scalar: (nth from: vs n: 2))",
            "0.36151162 0.4412291 0.37725854",
        );
    }

    #[test]
    pub fn test_prng_state_struct() {
        let mut prng = PrngStateStruct::new(542, 0.0, 1.0);

        for _ in 0..100 {
            let f = prng.prng_f32();
            println!("{}", f);
        }

        assert_eq!(prng.prng_f32(), 0.7896869);
    }

}
