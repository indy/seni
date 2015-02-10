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
    mode: "scheme",
    autoMatchParens: true
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

  (rect x: 500 y: 500 width: 1000 height: 1000 
   colour: (rgbColour r: 1.0 g: 1.0 b: 1.0 a: 1.0))

  (rect x: 250 y: 500 width: 200 height: 500 
   colour: (rgbColour r: 1.0 g: 1.0 b: 0.0 a: 1.0))

  (rect x: 500 y: 500 width: 200 height: 500 
   colour: (rgbColour r: 1.0 g: 1.0 b: 0.0 a: 0.5))
  
  (rect x: 750 y: 500 width: 200 height: 500 
   colour: (rgbColour r: 1.0 g: 1.0 b: 0.0 a: 0.2))
  
 (rect x: 500 y: 250 width: 1000 height: 200 
   colour: (rgbColour r: 1.0 g: 0.0 b: 0.0 a: 0.5))

  (rect x: 500 y: 750 width: 1000 height: 200 
   colour: (rgbColour r: 1.0 g: 0.0 b: 0.0 a: 1.0))

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





// another good wash background:
(define wash (fn (c: 60 m: 2.2)
		(loop (j from: (/ c 2) to: 1100 step: c)
			(loop (i from: (/ c 2) to: 1100 step: c) 
   				(onMatrixStack
                	(translate x: j y: i)
   	         		(rotate angle: (/ 3.14 (+ 0.26 (* 0.2 (perlin)))))
   	         		(rect width: (* c m) height: (* c m)
       	    			colour: (rgbColour r: (+ 0.5 (* 0.1 (perlin))) 
            	        	               g: (+ 0.5 (* 0.1 (perlin))) 
            	            	           b: (+ 0.5 (* 0.3 (perlin)))
            	                	       a: 0.9)))))))

(wash)




; a decent replication of the original clj wash
(define vary (fn (x: 0 y: 0 z: 0 scale: 1)
               (+ y (* scale (perlin x: x y: y z: z)))))


(define wash (fn (variation: 200
                      lineWidth: 70
                      lineSegments: 5
                      seed: 200)
               (loop (h from: -20 to: 1020 step: 20)
        (bezier tesselation: lineSegments
                lineWidth: lineWidth
                coords: (pair 0 (vary x: 0.10 y: h z: seed scale: variation)
                              333 (vary x: 333.33 y: h z: seed scale: variation)
                              666 (vary x: 666.66 y: h z: seed scale: variation)
                              1000 (vary x: 1000.10 y: h z: seed scale: variation))
                colour: (rgbColour r: 0.6 g: 0.6 b: 0.6 a: 0.4))
        (bezier tesselation: lineSegments
                lineWidth: lineWidth
                coords: (pair (vary x: 0.10 y: h z: seed scale: variation) 0
                              (vary x: 333.33 y: h z: seed scale: variation) 333
                              (vary x: 666.66 y: h z: seed scale: variation) 666
                              (vary x: 1000.10 y: h z: seed scale: variation) 1000)
                colour: (rgbColour r: 0.6 g: 0.6 b: 0.6 a: 0.4)))))


(wash)





(define vary (fn (x: 0 y: 0 z: 0 scale: 1)
               (+ y (* scale (perlin2 x: x y: y z: z)))))


(define wash (fn (variation: 200
                      lineWidth: 70
                      lineSegments: 5
                      seed: 272)
               (loop (h from: -20 to: 1020 step: 20)
        (bezier tesselation: lineSegments
                lineWidth: lineWidth
                coords: (pair 0 (vary x: 0.10 y: h z: seed scale: variation)
                              333 (vary x: 333.33 y: h z: seed scale: variation)
                              666 (vary x: 666.66 y: h z: seed scale: variation)
                              1000 (vary x: 1000.10 y: h z: seed scale: variation))
                colour: (rgbColour r: 0.15 g: 0.15 b: 0.15 a: 0.1))
        (bezier tesselation: lineSegments
                lineWidth: lineWidth
                coords: (pair (vary x: 0.10 y: h z: seed scale: variation) 0
                              (vary x: 333.33 y: h z: seed scale: variation) 333
                              (vary x: 666.66 y: h z: seed scale: variation) 666
                              (vary x: 1000.10 y: h z: seed scale: variation) 1000)
                colour: (rgbColour r: 0.15 g: 0.15 b: 0.15 a: 0.1)))))


  (wash)



*/
