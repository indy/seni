import PublicBinding from '../lang/PublicBinding';
import Util from './Util';

var MatrixStackFns = {

  pushMatrix: new PublicBinding(
    'pushMatrix',
    ``,
    {},
    (self, matrixStack) => {
      return () => matrixStack.pushMatrix();
    }
  ),

  popMatrix: new PublicBinding(
    'popMatrix',
    ``,
    {},
    (self, matrixStack) => {
      return () => matrixStack.popMatrix();
    }
  ),

  scale: new PublicBinding(
    'scale',
    ``,
    {x: 1, y: 1},
    (self, matrixStack) => {
      return (params) => {
        let {x, y} = Util.merge(params, self.defaults);
        return matrixStack.scale(x, y);
      };
    }
  ),

  translate: new PublicBinding(
    'translate',
    ``,
    {x: 0.0, y: 0.0},
    (self, matrixStack) => {
      return (params) => {
        let {x, y} = Util.merge(params, self.defaults);
        return matrixStack.translate(x, y);
      };
    }
  ),

  rotate: new PublicBinding(
    'rotate',
    ``,
    {angle: 0.0},
    (self, matrixStack) => {
      return (params) => {
        let {angle} = Util.merge(params, self.defaults);
        return matrixStack.rotate(angle);
      };
    }
  )
};


export default MatrixStackFns;
