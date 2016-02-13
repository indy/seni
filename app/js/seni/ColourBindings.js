/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

import PublicBinding from './PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';

const Format = Colour.Format;

// sets a value for a colour component,
// converting the colour to the right format if needed
function colourSet(publicBinding, format, component) {
  return params => {
    const {colour, value} = publicBinding.mergeWithDefaults(params);
    return Colour.setComponent(Colour.cloneAs(colour, format),
                               component, value);
  };
}

// gets a value for a colour component,
// converting the colour to the right format if needed
function colourGet(publicBinding, format, component) {
  return params => {
    const {colour} = publicBinding.mergeWithDefaults(params);
    return Colour.getComponent(Colour.cloneAs(colour, format),
                               component);
  };
}

const publicBindings = [
  new PublicBinding(
    'col/set-rgb-r',
    {
      description: `sets red component of the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.RGB, Colour.R)
  ),

  new PublicBinding(
    'col/get-rgb-r',
    {
      description: `gets red component of the given colour`,
      args: [['colour', 'Colour.defaultColour']],
      returns: 'the red component of the colour'
    },
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.RGB, Colour.R)
  ),

  new PublicBinding(
    'col/set-rgb-g',
    {
      description: `sets green component of the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.RGB, Colour.G)
  ),

  new PublicBinding(
    'col/get-rgb-g',
    {
      description: `gets green component of the given colour`,
      args: [['colour', 'Colour.defaultColour']],
      returns: 'the green component of the colour'
    },
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.RGB, Colour.G)
  ),

  new PublicBinding(
    'col/set-rgb-b',
    {
      description: `sets blue component of the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.RGB, Colour.B)
  ),

  new PublicBinding(
    'col/get-rgb-b',
    {
      description: `gets blue component of the given colour`,
      args: [['colour', 'Colour.defaultColour']],
      returns: 'the blue component of the colour'
    },
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.RGB, Colour.B)
  ),

  new PublicBinding(
    'col/set-alpha',
    {
      description: `sets alpha component of the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => params => {
      const {colour, value} = self.mergeWithDefaults(params);
      return Colour.setComponent(colour, Colour.ALPHA, value);
    }
  ),

  new PublicBinding(
    'col/get-alpha',
    {
      description: `gets alpha component of the given colour`,
      args: [['colour', 'Colour.defaultColour']],
      returns: 'the alpha component of the colour'
    },
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.getComponent(colour, Colour.ALPHA);
    }
  ),

  new PublicBinding(
    'col/set-lab-l',
    {
      description: `sets lab l component of the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.LAB, Colour.L)
  ),

  new PublicBinding(
    'col/get-lab-l',
    {
      description: `gets lab l component of the given colour`,
      args: [['colour', 'Colour.defaultColour']],
      returns: 'the lab l component of the colour'
    },
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.LAB, Colour.L)
  ),

  new PublicBinding(
    'col/set-lab-a',
    {
      description: `sets lab a component of the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.LAB, Colour.A)
  ),

  new PublicBinding(
    'col/get-lab-a',
    {
      description: `gets lab a component of the given colour`,
      args: [['colour', 'Colour.defaultColour']],
      returns: 'the lab a component of the colour'
    },
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.LAB, Colour.A)
  ),

  new PublicBinding(
    'col/set-lab-b',
    {
      description: `sets lab b component of the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => colourSet(self, Format.LAB, Colour.B)
  ),

  new PublicBinding(
    'col/get-lab-b',
    {
      description: `gets lab b component of the given colour`,
      args: [['colour', 'Colour.defaultColour']],
      returns: 'the lab b component of the colour'
    },
    {colour: Colour.defaultColour},
    self => colourGet(self, Format.LAB, Colour.B)
  ),

  new PublicBinding(
    'col/rgb',
    {
      description: `creates a colour given r, g, b and alpha values`,
      args: [['r', '0..1'],
             ['g', '0..1'],
             ['b', '0..1'],
             ['alpha', '0..1']],
      returns: 'the colour'
    },
    {r: 1.0, g: 0.1, b: 0.2, alpha: 0.5},
    self => params => {
      const {r, g, b, alpha} = self.mergeWithDefaults(params);
      return Colour.construct(Format.RGB, [r, g, b, alpha]);
    }
  ),

  new PublicBinding(
    'col/hsl',
    {
      description: `creates a colour given h, s, l and alpha values`,
      args: [['h', '0..360'],
             ['s', '0..1'],
             ['l', '0..1'],
             ['alpha', '0..1']],
      returns: 'the colour'
    },
    {h: 180.0, s: 0.1, l: 0.2, alpha: 0.5},
    self => params => {
      const {h, s, l, alpha} = self.mergeWithDefaults(params);
      const normalisedH = h % 360;
      return Colour.construct(Format.HSL, [normalisedH, s, l, alpha]);
    }
  ),

  new PublicBinding(
    'col/lab',
    {
      description: `creates a colour given l, a, b and alpha values`,
      args: [['l', '0..'],
             ['a', '-1..1'],
             ['b', '-1..1'],
             ['alpha', '0..1']],
      returns: 'the colour'
    },
    {l: 1.0, a: 0.1, b: 0.2, alpha: 0.5},
    self => params => {
      const {l, a, b, alpha} = self.mergeWithDefaults(params);
      return Colour.construct(Format.LAB, [l, a, b, alpha]);
    }
  ),

  new PublicBinding(
    'col/hsv',
    {
      description: `creates a colour given h, s, v and alpha values`,
      args: [['h', '0..360'],
             ['s', '0..1'],
             ['v', '0..1'],
             ['alpha', '0..1']],
      returns: 'the colour'
    },
    {h: 180.0, s: 0.1, v: 0.2, alpha: 0.5},
    self => params => {
      const {h, s, v, alpha} = self.mergeWithDefaults(params);
      const normalisedH = h % 360;
      return Colour.construct(Format.HSV, [normalisedH, s, v, alpha]);
    }
  ),

  new PublicBinding('RGB',
                    {description: ``, args: [], returns: ''},
                    {}, () => Format.RGB),
  new PublicBinding('HSL',
                    {description: ``, args: [], returns: ''},
                    {}, () => Format.HSL),
  new PublicBinding('LAB',
                    {description: ``, args: [], returns: ''},
                    {}, () => Format.LAB),
  new PublicBinding('HSV',
                    {description: ``, args: [], returns: ''},
                    {}, () => Format.HSV),

  new PublicBinding('white',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [1, 1, 1, 1])),
  new PublicBinding('black',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [0, 0, 0, 1])),
  new PublicBinding('red',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [1, 0, 0, 1])),
  new PublicBinding('green',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [0, 1, 0, 1])),
  new PublicBinding('blue',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [0, 0, 1, 1])),
  new PublicBinding('yellow',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [1, 1, 0, 1])),
  new PublicBinding('magenta',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [1, 0, 1, 1])),
  new PublicBinding('cyan',
                    {description: ``, args: [], returns: ''}, {},
                    () => Colour.construct(Format.RGB, [0, 1, 1, 1])),

  new PublicBinding(
    'col/convert',
    {
      description: `converts a colour to a different internal format`,
      args: [['format', 'the format to change to (RGB)'],
             ['colour', 'the input colour']],
      returns: 'the new colour'
    },
    {format: Format.RGB, colour: Colour.defaultColour},
    self => params => {
      const {format, colour} = self.mergeWithDefaults(params);
      return Colour.cloneAs(colour, format);
    }
  ),

  new PublicBinding(
    'col/complementary',
    {
      description: `calculate a complementary colour`,
      args: [['colour', 'the input colour']],
      returns: 'the complementary colour'
    },
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.complementary(colour);
    }
  ),

  new PublicBinding(
    'col/split-complementary',
    {
      description: `calculate split-complementary colours`,
      args: [['colour', 'the input colour']],
      returns: 'the 2 split complementary colours'
    },
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.splitComplementary(colour);
    }
  ),

  new PublicBinding(
    'col/analagous',
    {
      description: `calculate analagous colours`,
      args: [['colour', 'the input colour']],
      returns: 'the 2 analagous colours'
    },
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.analagous(colour);
    }
  ),

  new PublicBinding(
    'col/triad',
    {
      description: `calculate triad colours`,
      args: [['colour', 'the input colour']],
      returns: 'the 2 triad colours'
    },
    {colour: Colour.defaultColour},
    self => params => {
      const {colour} = self.mergeWithDefaults(params);
      return Colour.triad(colour);
    }
  ),

  new PublicBinding(
    'col/procedural-fn',
    {
      description: `calculate procedural colours`,
      args: [['a', '[0.5, 0.5, 0.5]'],
             ['b', '[0.5, 0.5, 0.5]'],
             ['c', '[0.5, 0.5, 0.5]'],
             ['d', '[0.5, 0.5, 0.5]'],
             ['alpha', '1.0']],
      returns: 'a function that accepts a t parameter'
    },
    {a: [0.5, 0.5, 0.5],
     b: [0.5, 0.5, 0.5],
     c: [1.0, 1.0, 1.0],
     d: [0.0, 0.33, 0.67],
     alpha: 1.0},
    self => params => {
      const {a, b, c, d, alpha} = self.mergeWithDefaults(params);
      return Colour.proceduralFn(a, b, c, d, alpha);
    }
  ),

  new PublicBinding(
    'col/bezier-fn',
    {
      description: `calculate colours using a Bezier function`,
      args: [['a', 'colour'],
             ['b', 'colour'],
             ['c', 'colour'],
             ['d', 'colour']],
      returns: 'a function that accepts a t parameter'
    },
    {a: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     b: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     c: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     d: Colour.construct(Format.RGB, [1, 1, 1, 1])},
    self => params => {
      const {a, b, c, d} = self.mergeWithDefaults(params);
      return Colour.bezierFn(a, b, c, d);
    }
  ),

  new PublicBinding(
    'col/quadratic-fn',
    {
      description: `calculate colours using a quadratic function`,
      args: [['a', 'colour'],
             ['b', 'colour'],
             ['c', 'colour']],
      returns: 'a function that accepts a t parameter'
    },
    {a: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     b: Colour.construct(Format.RGB, [1, 1, 1, 1]),
     c: Colour.construct(Format.RGB, [1, 1, 1, 1])},
    self => params => {
      const {a, b, c} = self.mergeWithDefaults(params);
      return Colour.quadraticFn(a, b, c);
    }
  ),

  new PublicBinding(
    'col/darken',
    {
      description: `darkens the given colour`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 1.0},
    self => params => {
      const {colour, value} = self.mergeWithDefaults(params);
      return Colour.setComponent(Colour.cloneAs(colour, Format.LAB),
                                 Colour.L, value);
    }
  ),

  new PublicBinding(
    'col/lighten',
    {
      description: `lightens the given colour by 'value'`,
      args: [['colour', 'Colour.defaultColour'],
             ['value', '0..1']],
      returns: 'the new colour'
    },
    {colour: Colour.defaultColour, value: 10.0},
    self => params => {
      const {colour, value} = self.mergeWithDefaults(params);
      const lab = Colour.cloneAs(colour, Format.LAB);
      const currentL = Colour.getComponent(lab, Colour.L);
      const newL = MathUtil.clamp(currentL + value, 0, 100);
      return Colour.setComponent(lab, Colour.L, newL);
    }
  )
];

export default {
  publicBindingType: 'binding',
  publicBindings
};
