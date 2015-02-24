import PublicBinding from '../lang/PublicBinding';

var MatrixStackFns = {

  pushMatrix: new PublicBinding(
    'pushMatrix',
    ``,
    (matrixStack) => {
      return () => matrixStack.pushMatrix();
    }
  ),

  popMatrix: new PublicBinding(
    'popMatrix',
    ``,
    (matrixStack) => {
      return () => matrixStack.popMatrix();
    }
  ),

  scale: new PublicBinding(
    'scale',
    ``,
    (matrixStack) => {
      return (params) => {
        let x = params.x || 1.0;
        let y = params.y || 1.0;
        return matrixStack.scale(x, y);
      };
    }
  ),

  translate: new PublicBinding(
    'translate',
    ``,
    (matrixStack) => {
      return (params) => {
        let x = params.x || 0.0;
        let y = params.y || 0.0;
        return matrixStack.translate(x, y);
      };
    }
  ),

  rotate: new PublicBinding(
    'rotate',
    ``,
    (matrixStack) => {
      return (params) => {
        let angle = params.angle || 0.0;
        return matrixStack.rotate(angle);
      };
    }
  )
};


export default MatrixStackFns;
