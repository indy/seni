
function bind(env, objs) {
  // env is an immutable Map
  // objs is an array of maps
  return objs.reduce((a, b) => a.merge(b), env);
}

export default bind;
