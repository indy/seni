import PublicBinding from '../lang/PublicBinding';

var BracketBindings = {
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
  )
};


export default BracketBindings;
