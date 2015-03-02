
var Util = {
  merge: function(obj, defaults) {
    let res = {};
    for(let p in defaults) {
      res[p] = obj[p] !== undefined ? obj[p] : defaults[p];
    }
    return res;
  }
};

export default Util;
