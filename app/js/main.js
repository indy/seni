import { Renderer } from 'seni/Renderer';
import { createEnv, evalForm } from 'lang/runtime';
import * as seniBind from 'seni/bind';

export function main() {
    var renderer = new Renderer("render-canvas");
    setupUI(renderer);

    // show something on the canvas at startup
    renderScript(renderer, initialCode());
}

function setupUI(renderer) {
  let d = document;

  let textArea = d.getElementById("codemirror-textarea");

  textArea.value = initialCode();

  var editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: true,
    mode: "scheme"
  });

  d.getElementById("eval-button").addEventListener("click", (e) => {
    renderScript(renderer, editor.getValue());
  });
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





(let ((c 30) (m 2.2))
  (loop (j from: (/ c 2) to: 1100 step: c)
    (loop (i from: (/ c 2) to: 1100 step: c) 
      (onMatrixStack 
        (translate x: i y: j)
        (rotate angle: (/ 3.14 4.1))
        (rect width: (* c m) height: (* c m)
              colour: (rgbColour r: (perlin) 
                                 g: (perlin) 
                                 b: (perlin)
                                 a: 0.7))))))

`;
}


/*


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




// a possible wash background:
(let ((c 30) (m 2.2))
  (loop (j from: (/ c 2) to: 1100 step: c)
        (loop (i from: (/ c 2) to: 1100 step: c) 
              (pushMatrix) 
               (translate x: i y: j)
               (rotate angle: (/ 3.14 0.29))
               (rect width: (* c m) height: (* c m)
                     colour: (rgbColour r: (perlin) 
                                        g: (perlin) 
                                        b: (perlin)
                                        a: 0.3))
              
             (popMatrix) 
              )))


// another good wash background:
(let ((c 30) (m 2.2))
  (loop (j from: (/ c 2) to: 1100 step: c)
        (loop (i from: (/ c 2) to: 1100 step: c) 
              (pushMatrix) 
               (translate x: j y: i)
               (rotate angle: (/ 3.14 (+ 0.26 (* 0.2 (perlin)))))
               (rect width: (* c m) height: (* c m)
                     colour: (rgbColour r: (+ 0.5 (* 0.1 (perlin))) 
                                        g: (+ 0.5 (* 0.1 (perlin))) 
                                        b: (+ 0.5 (* 0.3 (perlin)))
                                        a: 0.9))
              
             (popMatrix) 
              )))



*/
