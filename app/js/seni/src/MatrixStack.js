

export class MatrixStack {

  constructor() {
    this.reset();
  }

  reset() {
    this.stack = [mat4.create()];
    this.out = vec3.create();   // only pay the cost of construction once
  }

  getHead() {
    let stack = this.stack;
    return stack[stack.length - 1];
  }

  pushMatrix() {
    let m = this.getHead();
    this.stack.push(mat4.clone(m));
  }
  
  popMatrix() {
    this.stack.pop();
  }

  scale(sx, sy) {
    let m = this.getHead();
    mat4.scale(m, m, [sx, sy, 1.0]);
  }

  translate(tx, ty) {
    let m = this.getHead();
    mat4.translate(m, m, [tx, ty, 0.0]);
  }

  rotate(a) {
    let m = this.getHead();
    mat4.rotateZ(m, m, a);
  }

  transformVector(v) {
    let m = this.getHead();
    return vec3.transformMat4(this.out, v, m);
  }
}
