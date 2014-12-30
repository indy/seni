import { Renderer } from 'seni/Renderer';
import { createEnv, evalForm } from 'lang/runtime';
import * as seniBind from 'seni/bind';

export function main() {
  var renderer = new Renderer("render-canvas");
  let env = createEnv();
  env = seniBind.addBindings(env, renderer);

  let form = "(bezier 15 20 (list (list 440 400) (list 533 700) (list 766 200) (list 900 500)) 0 1)";

  renderer.preDrawScene();
  evalForm(env, form);
  renderer.postDrawScene();
}
