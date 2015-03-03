import Renderer from './seni/Renderer';
import Runtime from './lang/Runtime';
import Bind from './seni/Bind';

function renderScript(renderer, form) {
  let env = Runtime.createEnv();
  env = Bind.addBindings(env, renderer);

  renderer.preDrawScene();
  Runtime.evalForm(env, form);
  renderer.postDrawScene();
}

function initialCode() {
  let code = ` (define accumulated-rect
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
  return code;
}


// execute the function and log the time that it takes
function withTiming(fn) {
  let before = new Date();
  fn();
  let after = new Date();
  let duration = after - before;
  console.log('Time: ' + duration + 'ms');
}

function setupUI(renderer) {
  let d = document;

  let textArea = d.getElementById('codemirror-textarea');

  textArea.value = initialCode();

  let editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: false,
    mode: 'scheme',
    autoMatchParens: true,
    extraKeys: {
      'Ctrl-E': function() {
        withTiming(() => renderScript(renderer, editor.getValue()));
        return false;
      }
    }});
}

const SeniWebApplication = {
  mainFn() {
    let renderer = new Renderer('render-canvas');
    setupUI(renderer);
    renderScript(renderer, initialCode());

    let r = 'hello';
    console.log(r);
    return r;
  }
};

export default SeniWebApplication;
