function file2moduleName(filePath) {
  var res = filePath.replace(/\\/g, '/')
  // module name should be relative to `app` and `tools` folder
      .replace(/.*\/app\/js\//, '')    
      .replace(/.*\/tools\//, '')
  // module name should not include `src`, `test`, `lib`
      .replace(/\/src\//, '/')
      .replace(/\/web\//, '/')
      .replace(/\/perf_tmp\//, '/')
      .replace(/\/lib\//, '/')
  // module name should not have a suffix
      .replace(/\.\w*$/, '');
  return res;
}
if (typeof module !== 'undefined') {
  module.exports = file2moduleName;
}
