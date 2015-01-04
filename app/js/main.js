import { Renderer } from 'seni/Renderer';
import { createEnv, evalForm } from 'lang/runtime';
import * as seniBind from 'seni/bind';

export function main() {

  var renderer = new Renderer("render-canvas");
  setupUI(renderer);
}


function setupUI(renderer) {
  let d = document;

  let textArea = d.getElementById("textarea");

  textArea.value = initialCode();

  var editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: true,
    mode: "scheme"
  });

  d.getElementById("my-button").addEventListener("click", (e) => {
    renderScript(renderer, editor.getValue());
  });

  // show something on the canvas at startup
  renderScript(renderer, editor.getValue());
}

function renderScript(renderer, form) {
  let env = createEnv();
  env = seniBind.addBindings(env, renderer);

  renderer.preDrawScene();
  evalForm(env, form);
  renderer.postDrawScene();
}

function initialCode() {
  return `
(scale x: 2.5 y: 2.5)
(rotate angle: (/ 3.14 4))
(translate x: 300)
(bezier tessellation: 35 
        lineWidth: 20 
        coords: (list (list -100    0)
                      (list  -30 -100)
                      (list   30  100)
                      (list  100    0)) 
        tStart: 0 
        tEnd: 1)`;
}
