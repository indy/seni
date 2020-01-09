// --------------------------------------------------------------------------------
// editor

const modeName = 'seni';

function defineSeniMode() {
  // return an instance of CodeMirror with Seni mode defined
  CodeMirror.defineMode(modeName, CodeMirrorSeni.seniMode);
  return CodeMirror;
}

const Editor = {
  createEditor: function(element, customConfig) {
    const codeMirrorSeniMode = defineSeniMode();
    const defaultConfig = {
      lineNumbers: false,
      mode: modeName,
      autoCloseBrackets: true,
      matchBrackets: true
    };
    const res = Object.assign({}, defaultConfig, customConfig);

    return codeMirrorSeniMode.fromTextArea(element, res);
  }
};
