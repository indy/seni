
// todo: setup an interpreter with the bracket bindings

var Genetic = {
  createPhenotypeFromInitialValues: function(genes) {
    let phenoArray = genes.map((g) => Immutable.Map({value: g.initialValue,
                                                     gensym: g.gensym}));
    return Immutable.List(phenoArray);
  },

  createPhenotypeFromGenes: function(genes, seed) {
    // todo: add an rng with the given seed to the env with the bracket bindings
    seed = seed + 11;
    // this code is wrong
    let phenoArray = genes.map((g) => Immutable.Map({value: g.initialValue,
                                                     gensym: g.gensym}));
    return Immutable.List(phenoArray);
  },

  bindPhenotypeToEnv: function(phenotype, env) {
    return phenotype.reduce((e, p) => e.set(p.get('gensym'), p.get('value')),
                            env);
  }
};


export default Genetic;
