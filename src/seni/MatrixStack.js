class MatrixStack {

  constructor() {
    this.reset();
  }

  reset() {
    this.stack = [mat4.create()];
    this.out = vec3.create();   // only pay the cost of construction once
  }

  getHead() {
    const stack = this.stack;
    return stack[stack.length - 1];
  }

  pushMatrix() {
    const m = this.getHead();
    this.stack.push(mat4.clone(m));
  }

  popMatrix() {
    this.stack.pop();
  }

  scale(sx, sy) {
    const r = mat4.create();
    mat4.scale(r, r, [sx, sy, 1.0]);

    const m = this.getHead();
    mat4.mul(m, m, r);
  }

  translate(tx, ty) {
    const r = mat4.create();
    mat4.translate(r, r, [tx, ty, 0.0]);

    const m = this.getHead();
    mat4.mul(m, m, r);
  }

  rotate(a) {
    const r = mat4.create();
    mat4.rotateZ(r, r, a);

    const m = this.getHead();
    mat4.mul(m, m, r);
  }

  transformVector(v) {
    const m = this.getHead();
    return vec3.transformMat4(this.out, v, m);
  }
}

export default MatrixStack;
