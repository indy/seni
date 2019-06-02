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

pub fn create_identity_matrix() -> [f32; 16] {
    let out: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    out
}

pub fn create_ortho_matrix(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [f32; 16] {
    let lr = 1.0 / (left - right);
    let bt = 1.0 / (bottom - top);
    let nf = 1.0 / (near - far);

    let out: [f32; 16] = [
        -2.0 * lr,
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 * bt,
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 * nf,
        0.0,
        (left + right) * lr,
        (top + bottom) * bt,
        (far + near) * nf,
        1.0,
    ];

    out
}

pub fn matrix_scale(mat: &mut [f32; 16], x: f32, y: f32, z: f32) {
    mat[0] *= x;
    mat[5] *= y;
    mat[10] *= z;
}

pub fn matrix_translate(mat: &mut [f32; 16], x: f32, y: f32, z: f32) {
    mat[12] += x;
    mat[13] += y;
    mat[14] += z;
}
