/*jslint ignore:start*/

const SeedRandom = {
  buildUnsigned: function(seedVal) {
    const seedrandom = Math.seedrandom;
    const saveable = seedrandom(seedVal, {state: true});
    return () => saveable();
  },

  buildSigned: function(seedVal) {
    const seedrandom = Math.seedrandom;
    const saveable = seedrandom(seedVal, {state: true});
    return () => (saveable() * 2.0) - 1.0;
  }
};

export default SeedRandom;
