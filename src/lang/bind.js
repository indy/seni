
function bind(env, objs) {
  // objs is an array
  // add every key,val pair in obj as a binding to env
  let bindAll = function(obj) {
    for(let key in obj) {
      env.add(key, obj[key]);
    }
  };

  objs.forEach(o => bindAll(o));
  return env;
}

export default bind;
