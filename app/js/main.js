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
(loop (h from: 0 to: 500 step: 90)
  (loop (w from: 0 to: 400 step: 90)
    (bezier tessellation: 35 
            lineWidth: 20 
            coords: (list (list (- 440 w) (+ h 400))
                          (list (- 533 w) (+ h 700))
                          (list (- 766 w) (+ h 200))
                          (list (- 900 w) (+ h 500))) 
            tStart: 0 
            tEnd: 1)))`;
}
