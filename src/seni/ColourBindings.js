import PublicBinding from '../lang/PublicBinding';
import Colour from './Colour';

const Format = Colour.Format;

var ColourBindings = {
  setAlpha: new PublicBinding(
    'setAlpha',
    `sets the alpha value of the given colour
    arguments: colour, alpha`,
    {colour: Colour.defaultColour,
     alpha: 1.0},
    (self) => {
      return (params) => {
        let {colour, alpha} = self.mergeWithDefaults(params);
        return Colour.setAlpha(colour, alpha);
      };
    }
  ),

  rgbColour: new PublicBinding(
    'rgbColour',
    ``,
    {r: 1.0, g: 0.1, b: 0.2, a: 0.5},
    (self) => {
      return function(params) {
        let {r, g, b, a} = self.mergeWithDefaults(params);
        return Colour.construct(Format.RGB, [r, g, b, a]);
      };
    }
  ),

  hslColour: new PublicBinding(
    'hslColour',
    ``,
    {h: 1.0, s: 0.1, l: 0.2, a: 0.5},
    (self) => {
      return function(params) {
        let {h, s, l, a} = self.mergeWithDefaults(params);
        return Colour.construct(Format.HSL, [h, s, l, a]);
      };
    }
  ),

  labColour: new PublicBinding(
    'labColour',
    ``,
    {l: 1.0, a: 0.1, b: 0.2, alpha: 0.5},
    (self) => {
      return function(params) {
        let {l, a, b, alpha} = self.mergeWithDefaults(params);
        return Colour.construct(Format.LAB, [l, a, b, alpha]);
      };
    }
  ),

  hsvColour: new PublicBinding(
    'hsvColour',
    ``,
    {h: 1.0, s: 0.1, v: 0.2, a: 0.5},
    (self) => {
      return function(params) {
        let {h, s, v, a} = self.mergeWithDefaults(params);
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

  colourConvert: new PublicBinding(
    'colourConvert',
    ``,
    {format: Format.RGB, colour: Colour.defaultColour},
    (self) => function(params) {
      let {format, colour} = self.mergeWithDefaults(params);
      return Colour.cloneAs(colour, format);
    }
  ),

  complementary: new PublicBinding(
    'complementary',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        let {colour} = self.mergeWithDefaults(params);
        return Colour.complementary(colour);
      };
    }
  ),

  splitComplementary: new PublicBinding(
    'splitComplementary',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        let {colour} = self.mergeWithDefaults(params);
        return Colour.splitComplementary(colour);
      };
    }
  ),

  analagous: new PublicBinding(
    'analagous',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        let {colour} = self.mergeWithDefaults(params);
        return Colour.analagous(colour);
      };
    }
  ),

  triad: new PublicBinding(
    'triad',
    ``,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        let {colour} = self.mergeWithDefaults(params);
        return Colour.triad(colour);
      };
    }
  )
};


export default ColourBindings;
