import { Renderer } from './Renderer';
import { createEnv, evalForm } from './lang/runtime';

document.addEventListener('polymer-ready', function() {

    let env = createEnv();
    let res = evalForm(env, "(+ 3 2)");
    console.log(res);

    var renderer = new Renderer("render-canvas");
    renderer.preDrawScene();
    renderer.drawScene();
    renderer.postDrawScene();
});
