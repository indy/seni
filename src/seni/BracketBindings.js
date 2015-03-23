import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';

const BracketBindings = {
  identity: new PublicBinding(
    'identity',
    `returns value
    arguments: value`,
    {value: 42},
    (self) => {
      return (params) => {
        const {value} = self.mergeWithDefaults(params);
        return value;
      };
    }
  ),

  int: new PublicBinding(
    'int',
    `returns an integer in the range min..max-1
    arguments: min max`,
    {min: 0, max: 100},
    (self, rng) => {
      // rng is a SeedRandom returning values in the range 0..1
      return (params) => {
        const {min, max} = self.mergeWithDefaults(params);
        return Number.parseInt(MathUtil.interpolate(min, max, rng()));
      };
    }
  ),

  scalar: new PublicBinding(
    'scalar',
    `returns a number in the range 0..1
    arguments: -`,
    {min: 0.0, max: 1.0},
    (self, rng) => {
      self = self;
      // rng is a SeedRandom returning values in the range 0..1
      return (params) => {
        const {min, max} = self.mergeWithDefaults(params);
        return MathUtil.interpolate(min, max, rng());
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
