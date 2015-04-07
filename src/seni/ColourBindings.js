/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import PublicBinding from '../lang/PublicBinding';
import Colour from './Colour';

const Format = Colour.Format;

// sets a value for a colour component,
// converting the colour to the right format if needed
function colourSet(publicBinding, format, component) {
  return (params) => {
    const {colour, value} = publicBinding.mergeWithDefaults(params);
    return Colour.setComponent(Colour.cloneAs(colour, format),
                               component, value);
  };
}

// gets a value for a colour component,
// converting the colour to the right format if needed
function colourGet(publicBinding, format, component) {
  return (params) => {
    const {colour} = publicBinding.mergeWithDefaults(params);
    return Colour.getComponent(Colour.cloneAs(colour, format),
                               component);
  };
}

const ColourBindings = {
  colSetRGBRed: new PublicBinding(
    'col/set-rgb-r',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    (self) => colourSet(self, Format.RGB, Colour.R)
  ),

  colGetRGBRed: new PublicBinding(
    'col/get-rgb-r',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    (self) => colourGet(self, Format.RGB, Colour.R)
  ),

  colSetRGBGreen: new PublicBinding(
    'col/set-rgb-g',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    (self) => colourSet(self, Format.RGB, Colour.G)
  ),

  colGetRGBGreen: new PublicBinding(
    'col/get-rgb-g',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    (self) => colourGet(self, Format.RGB, Colour.G)
  ),

  colSetRGBBlue: new PublicBinding(
    'col/set-rgb-b',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    (self) => colourSet(self, Format.RGB, Colour.B)
  ),

  colGetRGBBlue: new PublicBinding(
    'col/get-rgb-b',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    (self) => colourGet(self, Format.RGB, Colour.B)
  ),

  colSetAlpha: new PublicBinding(
    'col/set-alpha',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    (self) => {
      return (params) => {
        const {colour, value} = self.mergeWithDefaults(params);
        return Colour.setComponent(colour, Colour.ALPHA, value);
      };
    }
  ),

  colGetAlpha: new PublicBinding(
    'col/get-alpha',
    `gets alpha component of the given colour`,
    {colour: Colour.defaultColour},
    (self) => {
      return (params) => {
        const {colour} = self.mergeWithDefaults(params);
        return Colour.getComponent(colour, Colour.ALPHA);
      };
    }
  ),

  colSetLABL: new PublicBinding(
    'col/set-lab-l',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    (self) => colourSet(self, Format.LAB, Colour.L)
  ),

  colGetLABL: new PublicBinding(
    'col/get-lab-l',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    (self) => colourGet(self, Format.LAB, Colour.L)
  ),

  colSetLABA: new PublicBinding(
    'col/set-lab-a',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    (self) => colourSet(self, Format.LAB, Colour.A)
  ),

  colGetLABA: new PublicBinding(
    'col/get-lab-a',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    (self) => colourGet(self, Format.LAB, Colour.A)
  ),

  colSetLABB: new PublicBinding(
    'col/set-lab-b',
    `sets red component of the given colour`,
    {colour: Colour.defaultColour, value: 1.0},
    (self) => colourSet(self, Format.LAB, Colour.B)
  ),

  colGetLABB: new PublicBinding(
    'col/get-lab-b',
    `gets red component of the given colour`,
    {colour: Colour.defaultColour},
    (self) => colourGet(self, Format.LAB, Colour.B)
  ),

  colRGB: new PublicBinding(
    'col/rgb',
    ``,
    {r: 1.0, g: 0.1, b: 0.2, alpha: 0.5},
    (self) => {
      return function(params) {
        const {r, g, b, alpha} = self.mergeWithDefaults(params);
        return Colour.construct(Format.RGB, [r, g, b, alpha]);
      };
    }
  ),

  colHSL: new PublicBinding(
    'col/hsl',
    ``,
    {h: 1.0, s: 0.1, l: 0.2, alpha: 0.5},
    (self) => {
      return function(params) {
        const {h, s, l, alpha} = self.mergeWithDefaults(params);
        return Colour.construct(Format.HSL, [h, s, l, alpha]);
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
    {h: 1.0, s: 0.1, v: 0.2, alpha: 0.5},
    (self) => {
      return function(params) {
        const {h, s, v, alpha} = self.mergeWithDefaults(params);
        return Colour.construct(Format.HSV, [h, s, v, alpha]);
      };
    }
  ),

  RGB: new PublicBinding('RGB', ``, {}, () => Format.RGB),
  HSL: new PublicBinding('HSL', ``, {}, () => Format.HSL),
  LAB: new PublicBinding('LAB', ``, {}, () => Format.LAB),
  HSV: new PublicBinding('HSV', ``, {}, () => Format.HSV),

  colConvert: new PublicBinding(
    'col/convert',
    ``,
    {format: Format.RGB, colour: Colour.defaultColour},
    (self) => {
      return function(params) {
        const {format, colour} = self.mergeWithDefaults(params);
        return Colour.cloneAs(colour, format);
      };
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
    'col/split-complementary',
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
