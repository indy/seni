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

  let editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: true,
    mode: "scheme",
    autoMatchParens: true,
    extraKeys: {
      "Ctrl-E": function(cm) {
        renderScript(renderer, editor.getValue());
        return false;
      }
    }});


  //d.getElementById("eval-button").addEventListener("click", (e) => {
  //  renderScript(renderer, editor.getValue());
  //});
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

(define accumulated-rect
  (fn (x: 0 
       y: 0 
       width: 10 
       height: 10 
       col: (rgbColour r: 0.0 g: 1.0 b: 0.0 a: 0.5) 
       volatility: 0)
      (let ((half-width (/ width 2))
            (half-height (/ height 2))
            (passes 10)
            (prngs (buildPRNG seed: "asdf")))
        (onMatrixStack
         (translate x: (+ x half-width) y: (+ y half-width))
         (loop (i to: 3)
               (let (((rr xr yr) (take num: 3 from: prngs)))
                 (onMatrixStack
                  (rotate angle: (* rr 0.02 volatility))
                  (translate x: (* xr 5 volatility) y: (* yr 5 volatility))
                  ; alpha = (/ col.a passes) 
                  (rect x: (- half-width) 
                        y: (- half-height) 
                        width: width 
                        height: height 
                        colour: col))))))))



(define bezier-stroked-rect
  (fn (x: 0 
       y: 0 
       width: 10 
       height: 10 
       col: (rgbColour r: 0.0 g: 1.0 b: 0.0 a: 0.5) 
       col-volatility: 0 ; todo
       volatility: 0
       overlap: 3
       iterations: 10
       seed: "shabba")

      (let ((th-width (/ width 3))
            (th-height (/ height 3))
            (vol volatility)
            
            (start-x (- x (/ width 2)))
            (start-y (- y (/ height 2)))
            
            (h-delta (/ height iterations))
            (h-strip-width (/ height iterations))
            (half-h-strip-width (/ h-strip-width 2))
            
            (v-delta (/ width iterations))
            (v-strip-width (/ width iterations))
            (half-v-strip-width (/ v-strip-width 2))
            
                                        ;[cr cg cb ca] (c/map-to-byte-range col)
            (half-alpha (/ ca 2))
            (prngs (buildPRNG seed: seed)))

        ;(rect x: x y: y width: width height: height
        ;      colour: (rgbColour r: 0.0 g: 1.0 b: 0.0 a: 0.5))
        
        ; horizontal strips
        (loop (i to: iterations)
              (let (((rx1 ry1 rx2 ry2 rx3 ry3 rx4 ry4) (take num: 8 from: prngs)))
                (bezier tessellation: 10
                        lineWidth: (+ overlap h-strip-width)
                        coords: (pair
                                 (+ (+ (* rx1 vol) start-x) (* 0 th-width))
                                 (+ (+ (* i h-delta) (* ry1 vol) start-y) half-h-strip-width)

                                 (+ (+ (* rx2 vol) start-x) (* 1 th-width))
                                 (+ (+ (* i h-delta) (* ry2 vol) start-y) half-h-strip-width)

                                 (+ (+ (* rx3 vol) start-x) (* 2 th-width))
                                 (+ (+ (* i h-delta) (* ry3 vol) start-y) half-h-strip-width)

                                 (+ (+ (* rx4 vol) start-x) (* 3 th-width))
                                 (+ (+ (* i h-delta) (* ry4 vol) start-y) half-h-strip-width))
                        colour: col)
                )
              )

        ; vertical strips
        (loop (i to: iterations)
              (let (((rx1 ry1 rx2 ry2 rx3 ry3 rx4 ry4) (take num: 8 from: prngs)))
                (bezier tessellation: 10
                        lineWidth: (+ overlap v-strip-width)
                        coords: (pair
                                 (+ (+ (* i v-delta) (* rx1 vol) start-x) half-v-strip-width)
                                 (+ (+ (* ry1 vol) start-y) (* 0 th-height))

                                 (+ (+ (* i v-delta) (* rx2 vol) start-x) half-v-strip-width)
                                 (+ (+ (* ry2 vol) start-y) (* 1 th-height))

                                 (+ (+ (* i v-delta) (* rx3 vol) start-x) half-v-strip-width)
                                 (+ (+ (* ry3 vol) start-y) (* 2 th-height))

                                 (+ (+ (* i v-delta) (* rx4 vol) start-x) half-v-strip-width)
                                 (+ (+ (* ry4 vol) start-y) (* 3 th-height)))
                        colour: col)
                )
              )

        )))


(accumulated-rect x: 750 y: 250 
                    width: 400 height: 400 
                    col: (rgbColour r: 0.8 g: 0.0 b: 0.0 a: 0.2)
                    volatility: 1.5)

(bezier-stroked-rect x: 250 y: 250 
                       width: 400 height: 400 
                       col: (rgbColour r: 0.8 g: 0.0 b: 0.0 a: 0.2)
                       volatility: 10.0
                       overlap: 3.0)

(bezier-stroked-rect x: 250 y: 750 
                       width: 400 height: 400 
                       col: (rgbColour r: 0.8 g: 0.0 b: 0.0 a: 0.2)
                       volatility: 10.0
                       overlap: 3.0)

(bezier-stroked-rect x: 750 y: 750 
                       width: 400 height: 400 
                       col: (rgbColour r: 0.8 g: 0.0 b: 0.0 a: 0.2)
                       volatility: 10.0
                       overlap: 3.0)

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
