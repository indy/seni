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
  let autogrow = d.getElementById("autogrow-textarea");  

  textArea.value = "(bezier 15 20 (list (list 440 400) (list 533 700) (list 766 200) (list 900 500)) 0 1)";
  autogrow.update();

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
