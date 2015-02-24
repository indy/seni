import PublicBinding from '../lang/PublicBinding';
import Colour from './Colour';
import ColourConstants from './ColourConstants';


const Format = ColourConstants.Format;

var ColourFns = {
  setAlpha: new PublicBinding(
    'setAlpha',
    `sets the alpha value of the given colour
    arguments: colour, alpha`,
    () => {
      let dc = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
      return (params) => {
        let colour = params.colour || dc;
        let alpha = params.alpha || 1.0;
        return colour.setAlpha(alpha);
      };
    }
  ),

  rgbColour: new PublicBinding(
    'rgbColour',
    ``,
    () => {
      return function(params) {
        let r = params.r || 1.0;
        let g = params.g || 0.5;
        let b = params.b || 0.5;
        let a = params.a || 1.0;
        return new Colour(Format.RGB, [r, g, b, a]);
      };
    }
  ),

  hslColour: new PublicBinding(
    'hslColour',
    ``,
    () => {
      return function(params) {
        let h = params.h || 1.0;
        let s = params.s || 0.5;
        let l = params.l || 0.5;
        let a = params.a || 1.0;

        return new Colour(Format.HSL, [h, s, l, a]);
      };
    }
  ),

  labColour: new PublicBinding(
    'labColour',
    ``,
    () => {
      return function(params) {
        let l = params.l || 1.0;
        let a = params.a || 0.5;
        let b = params.b || 0.5;
        let alpha = params.alpha || 1.0;

        return new Colour(Format.LAB, [l, a, b, alpha]);
      };
    }
  ),

  hsvColour: new PublicBinding(
    'hsvColour',
    ``,
    () => {
      return function(params) {
        let h = params.h || 1.0;
        let s = params.s || 0.5;
        let v = params.v || 0.5;
        let a = params.a || 1.0;

        return new Colour(Format.HSV, [h, s, v, a]);
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
        return;
      }
      let format = params.format || Format.RGB;
      let colour = params.colour;
      return colour.cloneAs(format);
    }
  ),

  complementary: new PublicBinding(
    'complementary',
    ``,
    () => {
      let dc = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
      return (params) => {
        let colour = params.colour || dc;
        return colour.complementary();
      };
    }
  ),

  splitComplementary: new PublicBinding(
    'splitComplementary',
    ``,
    () => {
      let dc = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
      return (params) => {
        let colour = params.colour || dc;
        return colour.splitComplementary();
      };
    }
  ),

  analagous: new PublicBinding(
    'analagous',
    ``,
    () => {
      let dc = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
      return (params) => {
        let colour = params.colour || dc;
        return colour.analagous();
      };
    }
  ),

  triad: new PublicBinding(
    'triad',
    ``,
    () => {
      let dc = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
      return (params) => {
        let colour = params.colour || dc;
        return colour.triad();
      };
    }
  )
};


export default ColourFns;
