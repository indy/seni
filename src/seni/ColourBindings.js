import PublicBinding from '../lang/PublicBinding';
import Colour from './Colour';

const Format = Colour.Format;

const ColourBindings = {
  colSetAlpha: new PublicBinding(
    'col/setAlpha',
    `sets the alpha value of the given colour
    arguments: colour, alpha`,
    {colour: Colour.defaultColour,
     alpha: 1.0},
    (self) => {
      return (params) => {
        const {colour, alpha} = self.mergeWithDefaults(params);
        return Colour.setAlpha(colour, alpha);
      };
    }
  ),

  colGetAlpha: new PublicBinding(
    'col/getAlpha',
    `get the alpha value of the given colour
    arguments: colour`,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        const {colour} = self.mergeWithDefaults(params);
        return Colour.getAlpha(colour);
      };
    }
  ),

  colSetLightness: new PublicBinding(
    'col/setLightness',
    `sets the alpha value of the given colour
    arguments: colour, alpha`,
    {colour: Colour.defaultColour,
     l: 1.0},
    (self) => {
      return (params) => {
        const {colour, l} = self.mergeWithDefaults(params);
        return Colour.setLightness(colour, l);
      };
    }
  ),

  colGetLightness: new PublicBinding(
    'col/getLightness',
    `get the alpha value of the given colour
    arguments: colour`,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        const {colour} = self.mergeWithDefaults(params);
        return Colour.getLightness(colour);
      };
    }
  ),

  colRGB: new PublicBinding(
    'col/rgb',
    ``,
    {r: 1.0, g: 0.1, b: 0.2, a: 0.5},
    (self) => {
      return function(params) {
        const {r, g, b, a} = self.mergeWithDefaults(params);
        return Colour.construct(Format.RGB, [r, g, b, a]);
      };
    }
  ),

  colHSL: new PublicBinding(
    'col/hsl',
    ``,
    {h: 1.0, s: 0.1, l: 0.2, a: 0.5},
    (self) => {
      return function(params) {
        const {h, s, l, a} = self.mergeWithDefaults(params);
        return Colour.construct(Format.HSL, [h, s, l, a]);
      };
    }
  ),

  colLAB: new PublicBinding(
    'col/lab',
    ``,
    {l: 1.0, a: 0.1, b: 0.2, alpha: 0.5},
    (self) => {
      return function(params) {
        const {l, a, b, alpha} = self.mergeWithDefaults(params);
        return Colour.construct(Format.LAB, [l, a, b, alpha]);
      };
    }
  ),

  colHSV: new PublicBinding(
    'col/hsv',
    ``,
    {h: 1.0, s: 0.1, v: 0.2, a: 0.5},
    (self) => {
      return function(params) {
        const {h, s, v, a} = self.mergeWithDefaults(params);
        return Colour.construct(Format.HSV, [h, s, v, a]);
      };
    }
  ),

  RGB: new PublicBinding(
    'RGB',
    ``,
    {},
    () => Format.RGB
  ),

  HSL: new PublicBinding(
    'HSL',
    ``,
    {},
    () => Format.HSL
  ),

  LAB: new PublicBinding(
    'LAB',
    ``,
    {},
    () => Format.LAB
  ),

  HSV: new PublicBinding(
    'HSV',
    ``,
    {},
    () => Format.HSV
  ),

  colConvert: new PublicBinding(
    'col/convert',
    ``,
    {format: Format.RGB, colour: Colour.defaultColour},
    (self) => function(params) {
      const {format, colour} = self.mergeWithDefaults(params);
      return Colour.cloneAs(colour, format);
    }
  ),

  colComplementary: new PublicBinding(
    'col/complementary',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        const {colour} = self.mergeWithDefaults(params);
        return Colour.complementary(colour);
      };
    }
  ),

  colSplitComplementary: new PublicBinding(
    'col/splitComplementary',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        const {colour} = self.mergeWithDefaults(params);
        return Colour.splitComplementary(colour);
      };
    }
  ),

  colAnalagous: new PublicBinding(
    'col/analagous',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        const {colour} = self.mergeWithDefaults(params);
        return Colour.analagous(colour);
      };
    }
  ),

  colTriad: new PublicBinding(
    'col/triad',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        const {colour} = self.mergeWithDefaults(params);
        return Colour.triad(colour);
      };
    }
  )
};


export default ColourBindings;
