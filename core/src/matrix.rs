// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub struct Matrix {
    m: [f32; 16],
}

impl Matrix {
    pub fn identity() -> Self {
        Matrix {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn copy_matrix(a: &Matrix) -> Self {
        Matrix { m: a.m }
    }

    pub fn copy_from(&mut self, a: &Matrix) {
        self.m = a.m;
    }

    // self = self * b
    pub fn multiply(&mut self, b: &Matrix) {
        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a03 = self.m[3];
        let a10 = self.m[4];
        let a11 = self.m[5];
        let a12 = self.m[6];
        let a13 = self.m[7];
        let a20 = self.m[8];
        let a21 = self.m[9];
        let a22 = self.m[10];
        let a23 = self.m[11];
        let a30 = self.m[12];
        let a31 = self.m[13];
        let a32 = self.m[14];
        let a33 = self.m[15];

        {
            let b0 = b.m[0];
            let b1 = b.m[1];
            let b2 = b.m[2];
            let b3 = b.m[3];

            self.m[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
            self.m[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
            self.m[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
            self.m[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
        }

        {
            let b0 = b.m[4];
            let b1 = b.m[5];
            let b2 = b.m[6];
            let b3 = b.m[7];
            self.m[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
            self.m[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
            self.m[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
            self.m[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
        }
        {
            let b0 = b.m[8];
            let b1 = b.m[9];
            let b2 = b.m[10];
            let b3 = b.m[11];
            self.m[8] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
            self.m[9] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
            self.m[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
            self.m[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
        }
        {
            let b0 = b.m[12];
            let b1 = b.m[13];
            let b2 = b.m[14];
            let b3 = b.m[15];
            self.m[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
            self.m[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
            self.m[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
            self.m[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
        }
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.m[0] *= x;
        self.m[1] *= x;
        self.m[2] *= x;
        self.m[3] *= x;

        self.m[4] *= y;
        self.m[5] *= y;
        self.m[6] *= y;
        self.m[7] *= y;

        self.m[8] *= z;
        self.m[9] *= z;
        self.m[10] *= z;
        self.m[11] *= z;
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.m[12] += self.m[0] * x + self.m[4] * y + self.m[8] * z;
        self.m[13] += self.m[1] * x + self.m[5] * y + self.m[9] * z;
        self.m[14] += self.m[2] * x + self.m[6] * y + self.m[10] * z;
        self.m[15] += self.m[3] * x + self.m[7] * y + self.m[11] * z;
    }

    pub fn rotate_z(&mut self, rad: f32) {
        let s = rad.sin();
        let c = rad.cos();
        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a03 = self.m[3];
        let a10 = self.m[4];
        let a11 = self.m[5];
        let a12 = self.m[6];
        let a13 = self.m[7];

        // Perform axis-specific matrix multiplication
        self.m[0] = a00 * c + a10 * s;
        self.m[1] = a01 * c + a11 * s;
        self.m[2] = a02 * c + a12 * s;
        self.m[3] = a03 * c + a13 * s;
        self.m[4] = a10 * c - a00 * s;
        self.m[5] = a11 * c - a01 * s;
        self.m[6] = a12 * c - a02 * s;
        self.m[7] = a13 * c - a03 * s;
    }

    pub fn transform_vec2(&self, x: f32, y: f32) -> (f32, f32) {
        let outx = self.m[0] * x + self.m[4] * y + self.m[12];
        let outy = self.m[1] * x + self.m[5] * y + self.m[13];
        (outx, outy)
    }

    pub fn transform_vec3(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let w1 = self.m[3] * x + self.m[7] * y + self.m[11] * z + self.m[15];
        let w = if w1 == 0.0 { 1.0 } else { w1 };

        let outx = (self.m[0] * x + self.m[4] * y + self.m[8] * z + self.m[12]) / w;
        let outy = (self.m[1] * x + self.m[5] * y + self.m[9] * z + self.m[13]) / w;
        let outz = (self.m[2] * x + self.m[6] * y + self.m[10] * z + self.m[14]) / w;

        (outx, outy, outz)
    }
}

pub struct MatrixStack {
    stack: Vec<Matrix>,
}

impl Default for MatrixStack {
    fn default() -> MatrixStack {
        let mut ms = MatrixStack {
            stack: Vec::with_capacity(16),
        };
        ms.reset();
        ms
    }
}

impl MatrixStack {
    pub fn reset(&mut self) {
        self.stack.clear();
        // add an identity matrix onto the stack so that further
        // scale/rotate/translate ops can work
        self.stack.push(Matrix::identity())
    }

    pub fn peek(&self) -> Option<&Matrix> {
        self.stack.last()
    }

    pub fn push(&mut self) {
        let mut head: Option<Matrix> = None;

        if let Some(top) = self.peek() {
            head = Some(Matrix::copy_matrix(top));
        }

        if let Some(m) = head {
            self.stack.push(m);
        }
    }

    pub fn pop(&mut self) -> Option<Matrix> {
        self.stack.pop()
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        let mut m = Matrix::identity();
        m.scale(sx, sy, 1.0);

        let len = self.stack.len();
        self.stack[len - 1].multiply(&m);
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        let mut m = Matrix::identity();
        m.translate(tx, ty, 0.0);

        let len = self.stack.len();
        self.stack[len - 1].multiply(&m);
    }

    pub fn rotate(&mut self, a: f32) {
        let mut m = Matrix::identity();
        m.rotate_z(a);

        let len = self.stack.len();
        self.stack[len - 1].multiply(&m);
    }

    // todo: should this return a Result? (and not the dodgy else clause)
    // is that too much of a performance hit?
    pub fn transform_vec2(&self, x: f32, y: f32) -> (f32, f32) {
        if let Some(top) = self.peek() {
            top.transform_vec2(x, y)
        } else {
            (x, y)
        }
    }

    // todo: should this return a Result? (and not the dodgy else clause)
    // is that too much of a performance hit?
    pub fn transform_vec3(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        if let Some(top) = self.peek() {
            top.transform_vec3(x, y, z)
        } else {
            (x, y, z)
        }
    }
}
