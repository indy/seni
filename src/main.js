import { Renderer } from './Renderer';

document.addEventListener('polymer-ready', function() {
  var renderer = new Renderer("render-canvas");
  renderer.preDrawScene();
  renderer.drawScene();
  renderer.postDrawScene();
});
