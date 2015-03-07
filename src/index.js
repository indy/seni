import Renderer from './seni/Renderer';
import Runtime from './lang/Runtime';
import Bind from './seni/Bind';

function renderScript(renderer, form) {
  let env = Runtime.createEnv();
  env = Bind.addBindings(env, renderer);

  renderer.preDrawScene();
  Runtime.evalForm(env, form);
  renderer.postDrawScene();
}

function initialCode() {
  let code = `(rect x: 500 y: 500)`;
  return code;
}


// execute the function and log the time that it takes
function withTiming(fn) {
  let before = new Date();
  fn();
  let after = new Date();
  let duration = after - before;
  console.log('Time: ' + duration + 'ms');
}

function setupUI(renderer) {
  let d = document;

  let textArea = d.getElementById('codemirror-textarea');

  textArea.value = initialCode();

  let editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: false,
    mode: 'scheme',
    autoMatchParens: true,
    extraKeys: {
      'Ctrl-E': function() {
        withTiming(() => renderScript(renderer, editor.getValue()));
        return false;
      }
    }});
}

const SeniWebApplication = {
  mainFn() {
    let renderer = new Renderer('render-canvas');
    setupUI(renderer);
    renderScript(renderer, initialCode());

    let r = 'hello';
    console.log(r);
    return r;
  }
};

export default SeniWebApplication;
