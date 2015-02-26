import PublicBinding from '../lang/PublicBinding';

const Core = {
  takeBinding: new PublicBinding(
    'take',

    `invokes the 'from' function 'num' times, returning a list`,

    () => function(params) {
      let num = params.num || 0;
      let from = params.from || function() { return 0;};

      let res = [];
      for(let i=0; i<num; i++) {
        res.push(from());
      }
      return res;
    }
  )
};



export default Core;
