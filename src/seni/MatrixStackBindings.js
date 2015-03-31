import PublicBinding from '../lang/PublicBinding';

const MatrixStackBindings = {

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
        const {x, y} = self.mergeWithDefaults(params);
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
        const {x, y} = self.mergeWithDefaults(params);
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
        const {angle} = self.mergeWithDefaults(params);
        return matrixStack.rotate(angle);
      };
    }
  )
};

export default MatrixStackBindings;
