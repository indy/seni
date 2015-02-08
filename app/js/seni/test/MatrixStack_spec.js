import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import {
  MatrixStack
} from 'seni/MatrixStack';

export function main() {

  describe('MatrixStack', () => {

    function matrixRowColumn(m, r, c) {
      return m[(c*4) + r];
    }

    function expectIdentity(m) {
      for(var j=0;j<4;j++) {
        for(var i=0;i<4;i++) {
          expect(matrixRowColumn(m, i, j)).toEqual(i === j ? 1 : 0);
        }
      }
    }

    var ms;
    
    beforeEach(() => {
      ms = new MatrixStack();
    });

    it('constructing', () => {
      expectIdentity(ms.getHead());
    });

    it('should scale', () => {
      ms.scale(10, 20);
      var m = ms.getHead();
      expect(matrixRowColumn(m, 0, 0)).toEqual(10);
      expect(matrixRowColumn(m, 1, 1)).toEqual(20);
    });

    it('should translate', () => {
      ms.translate(30, 40);
      var m = ms.getHead();
      expect(matrixRowColumn(m, 0, 3)).toEqual(30);
      expect(matrixRowColumn(m, 1, 3)).toEqual(40);
    });

    it('should rotate', () => {
      ms.translate(20, 0);
      ms.rotate(0.5);
      var m = ms.getHead();
      // todo: write a test
    });


    it('should push and pop', () => {

      ms.translate(30, 40);
      var m = ms.getHead();
      expect(matrixRowColumn(m, 0, 3)).toEqual(30);
      expect(matrixRowColumn(m, 1, 3)).toEqual(40);

      ms.pushMatrix()

      ms.scale(10, 20);
      m = ms.getHead();
      expect(matrixRowColumn(m, 0, 3)).toEqual(30);
      expect(matrixRowColumn(m, 1, 3)).toEqual(40);
      expect(matrixRowColumn(m, 0, 0)).toEqual(10);
      expect(matrixRowColumn(m, 1, 1)).toEqual(20);

      ms.popMatrix()
      m = ms.getHead();
      expect(matrixRowColumn(m, 0, 3)).toEqual(30);
      expect(matrixRowColumn(m, 1, 3)).toEqual(40);
      expect(matrixRowColumn(m, 0, 0)).toEqual(1);
      expect(matrixRowColumn(m, 1, 1)).toEqual(1);
    });
  });
}
