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

(let ((c 50))
  (loop (j from: (/ c 2) to: 1000 step: c)
        (loop (i from: (/ c 2) to: 1000 step: c) 
              (rect width: c height: c
                    x: i y: j
                    colour: (rgbColour r: (perlin) 
                                       g: (perlin) 
                                       b: (perlin)
                             a: 0.5)))))
`;
}


/*

(let ((c 50))
  (loop (j from: (/ c 2) to: 1000 step: c)
        (loop (i from: (/ c 2) to: 1000 step: c) 
              (rect width: c height: c
                    x: i y: j
                    colour: (rgbColour r: (perlin) 
                                       g: (perlin) 
                                       b: (perlin)
                             a: 0.5)))))


 

(rect x: 500 y: 500 width: 200 height: 200 
      colour: (rgbColour r: (perlin) 
                         g: (perlin) 
                         b: (perlin)))
(scale x: 2.5 y: 2.5)
(rotate angle: (/ 3.14 4))
(translate x: 300)
(rect width: 200 height: 200 
      colour: (rgbColour r: (perlin) 
                         g: (perlin) 
                         b: (perlin)
                         a: 0.5))
(bezierBulging tessellation: 35 
        lineWidth: 20
        coords: (pair -100    0
                       -30 -100
                        30  100
                       100    0)
        tStart: 0 
        tEnd: 1
        colour: (rgbColour r: 0.5 
                           g: 0.1 
                           b: 1.0
                           a: 0.4))










(scale x: 2.5 y: 2.5)
(rotate angle: (/ 3.14 4))
(translate x: 300)
(rect width: 200 height: 200 
      colour: (rgbColour r: (perlin) 
                         g: (perlin) 
                         b: (perlin)
                         a: 0.5))




(rect x: 500 y: 500 width: 200 height: 200 colour: (rgbColour r: (perlin) g: (perlin) b: (perlin)))
(scale x: 2.5 y: 2.5)
(rotate angle: (/ 3.14 4))
(translate x: 300)
(rect width: 200 height: 200 colour: (rgbColour r: (perlin) g: (perlin) b: (perlin)))
(bezierBulging tessellation: 35 
        lineWidth: 20
        coords: (pair -100    0
                       -30 -100
                        30  100
                       100    0)
        tStart: 0 
        tEnd: 1
        colour: (rgbColour r: 0.5 g: 0.1 b: 1.0))




(let ((c 50) (m 1.2))
  (loop (j from: (/ c 2) to: 1000 step: c)
        (loop (i from: (/ c 2) to: 1000 step: c) 
              (pushMatrix)
              (translate x: i y: j)
              (rotate angle: (/ 3.14 4))
              (rect width: (* c m) height: (* c m)
                    colour: (rgbColour r: (perlin) 
                                       g: (perlin) 
                                       b: (perlin)
                             a: 0.7))
              (popMatrix))))


                
(let ((c 50) (m 1.2))
  (loop (j from: (/ c 2) to: 1000 step: c)
        (loop (i from: (/ c 2) to: 1000 step: c) 
              (pushMatrix) 
               (translate x: i y: j)
               (rotate angle: (/ 3.14 4))
               (rect width: (* c m) height: (* c m)
                     colour: (rgbColour r: (perlin) 
                                        g: (perlin) 
                                        b: (perlin)
                                        a: 0.7))
              
             (popMatrix) 
              )))                */
