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
  //let autogrow = d.getElementById("autogrow-textarea");  

  textArea.value = "(loop (h from: 0 to: 500 step: 90)(loop (w from: 0 to: 400 step: 90)(bezier tessellation: 35 lineWidth: 20 coords: (list (list (- 440 w) (+ h 400))(list (- 533 w) (+ h 700))(list (- 766 w) (+ h 200))(list (- 900 w) (+ h 500))) tStart: 0 tEnd: 1)))";

  //autogrow.update();

  d.getElementById("my-button").addEventListener("click", (e) => {
    renderScript(renderer, textArea.value);
  });
}

function renderScript(renderer, form) {
  let env = createEnv();
  env = seniBind.addBindings(env, renderer);

  renderer.preDrawScene();
  evalForm(env, form);
  renderer.postDrawScene();
}


/*
(loop (h from: 0 to: 500 step: 90)
  (loop (w from: 0 to: 400 step: 90)
    (bezier tessellation: 35
               lineWidth: 20
               coords: (list (list (- 440 w) (+ h 400))
                                 (list (- 533 w) (+ h 700))
                                 (list (- 766 w) (+ h 200))
                                 (list (- 900 w) (+ h 500)))
               tStart: 0
               tEnd: 1)))



*/
