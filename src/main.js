import { Renderer } from './Renderer';

document.addEventListener('polymer-ready', function() {
  var navicon = document.getElementById('navicon');
  var drawerPanel = document.getElementById('drawerPanel');
  navicon.addEventListener('click', function() {
    drawerPanel.togglePanel();
  });

  var renderer = new Renderer("render-canvas");
  renderer.preDrawScene();
  renderer.drawScene();
  renderer.postDrawScene();
});
