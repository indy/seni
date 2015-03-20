import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';

let BracketBindings = {
  identity: new PublicBinding(
    'identity',
    `returns value
    arguments: value`,
    {value: 42},
    (self) => {
      return (params) => {
        let {value} = self.mergeWithDefaults(params);
        return value;
      };
    }
  ),

  inRange: new PublicBinding(
    'inRange',
    `returns an integer in the range min..max-1
    arguments: min max`,
    {min: 0, max: 100},
    (self, rng) => {
      // rng is a SeedRandom returning values in the range 0..1
      return (params) => {
        let {min, max} = self.mergeWithDefaults(params);
        return Number.parseInt(MathUtil.interpolate(min, max, rng()));
      };
    }
  ),

  scalar: new PublicBinding(
    'scalar',
    `returns a number in the range 0..1
    arguments: -`,
    {},
    (self, rng) => {
      self = self;
      // rng is a SeedRandom returning values in the range 0..1
      return () => {
        return rng();
      };
    }
  ),

  testPlus: new PublicBinding(
    'testPlus',
    `[FOR TESTING ONLY] returns + character
    arguments: -`,
    {},
    () => {
      // rng is a SeedRandom returning values in the range 0..1
      return () => {
        return '+';
      };
    }
  )
};


export default BracketBindings;
