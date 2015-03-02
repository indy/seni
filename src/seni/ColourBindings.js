import PublicBinding from '../lang/PublicBinding';
import Colour from './Colour';

const Format = Colour.Format;

var ColourBindings = {
  setAlpha: new PublicBinding(
    'setAlpha',
    `sets the alpha value of the given colour
    arguments: colour, alpha`,
    () => {
      let dc = Colour.construct(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
      return (params) => {
        let colour = params.colour || dc;
        let alpha = params.alpha !== undefined ? params.alpha : 1.0;
        return Colour.setAlpha(colour, alpha);
      };
    }
  ),

  rgbColour: new PublicBinding(
    'rgbColour',
    ``,
    () => {
      return function(params) {
        let r = params.r !== undefined ? params.r : 1.0;
        let g = params.g !== undefined ? params.g : 0.1;
        let b = params.b !== undefined ? params.b : 0.2;
        let a = params.a !== undefined ? params.a : 0.5;
        return Colour.construct(Format.RGB, [r, g, b, a]);
      };
    }
  ),

  hslColour: new PublicBinding(
    'hslColour',
    ``,
    () => {
      return function(params) {
        let h = params.h !== undefined ? params.h : 1.0;
        let s = params.s !== undefined ? params.s : 0.1;
        let l = params.l !== undefined ? params.l : 0.2;
        let a = params.a !== undefined ? params.a : 0.5;

        return Colour.construct(Format.HSL, [h, s, l, a]);
      };
    }
  ),

  labColour: new PublicBinding(
    'labColour',
    ``,
    () => {
      return function(params) {
        let l = params.l !== undefined ? params.l : 1.0;
        let a = params.a !== undefined ? params.a : 0.1;
        let b = params.b !== undefined ? params.b : 0.2;
        let alpha = params.alpha !== undefined ? params.alpha : 0.5;

        return Colour.construct(Format.LAB, [l, a, b, alpha]);
      };
    }
  ),

  hsvColour: new PublicBinding(
    'hsvColour',
    ``,
    () => {
      return function(params) {
        let h = params.h !== undefined ? params.h : 1.0;
        let s = params.s !== undefined ? params.s : 0.1;
        let v = params.v !== undefined ? params.v : 0.2;
        let a = params.a !== undefined ? params.a : 0.5;

        return Colour.construct(Format.HSV, [h, s, v, a]);
      };
    }
  ),

  RGB: new PublicBinding(
    'RGB',
    ``,
    () => Format.RGB
  ),

  HSL: new PublicBinding(
    'HSL',
    ``,
    () => Format.HSL
  ),

  LAB: new PublicBinding(
    'LAB',
    ``,
    () => Format.LAB
  ),

  HSV: new PublicBinding(
    'HSV',
    ``,
    () => Format.HSV
  ),

  colourConvert: new PublicBinding(
    'colourConvert',
    ``,
    () => function(params) {
      if(params.colour === undefined) {
        return Colour.defaultColour;
      }
      let format = params.format || Format.RGB;
      let colour = params.colour;
      return Colour.cloneAs(colour, format);
    }
  ),

  complementary: new PublicBinding(
    'complementary',
    ``,
    () => {
      return (params) => {
        let colour = params.colour || Colour.defaultColour;
        return Colour.complementary(colour);
      };
    }
  ),

  splitComplementary: new PublicBinding(
    'splitComplementary',
    ``,
    () => {
      return (params) => {
        let colour = params.colour || Colour.defaultColour;
        return Colour.splitComplementary(colour);
      };
    }
  ),

  analagous: new PublicBinding(
    'analagous',
    ``,
    () => {
      return (params) => {
        let colour = params.colour || Colour.defaultColour;
        return Colour.analagous(colour);
      };
    }
  ),

  triad: new PublicBinding(
    'triad',
    ``,
    () => {
      return (params) => {
        let colour = params.colour || Colour.defaultColour;
        return Colour.triad(colour);
      };
    }
  )
};


export default ColourBindings;
