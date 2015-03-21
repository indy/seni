import PublicBinding from '../lang/PublicBinding';

const Core = {
  takeBinding: new PublicBinding(
    'take',

    `invokes the 'from' function 'num' times, returning a list`,

    {num: 1, from: function() {return 0;}},

    (self) => function(params) {
      const {num, from} = self.mergeWithDefaults(params);
      const res = [];
      for(let i=0; i<num; i++) {
        res.push(from());
      }
      return res;
    }
  )
};



export default Core;
