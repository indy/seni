

/*
document.addEventListener('DOMContentLoaded', function () {
    console.log('Aloha');
});
*/

import { Renderer } from 'seni/Renderer';
import { createEnv, evalForm } from 'lang/runtime';

export function main() {
    console.log("hello world");

    let env = createEnv();
    let res = evalForm(env, "(+ 3 2)");
    console.log(res);

    var renderer = new Renderer("render-canvas");
    renderer.preDrawScene();
    renderer.drawScene();
    renderer.postDrawScene();
}
