import Renderer from './seni/Renderer';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';
import Bind from './seni/Bind';
import Trivia from './seni/Trivia';

function renderScript(renderer, form) {
  let env = Runtime.createEnv();
  env = Bind.addBindings(env, renderer);

  renderer.preDrawScene();
  const ast = Runtime.buildAst(env, form);

  const traits = Genetic.buildTraits(ast);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  Runtime.evalAst(env, ast, genotype);

  renderer.postDrawScene();
}

function initialCode() {
  const code = `(define numSquaresToRender [15 (inRange min: 2 max: 20)])
(define gapSize [30 (inRange min: 5 max: 50)])

(define numSquares (+ 2 numSquaresToRender))

(define numGaps (+ numSquares 1))
(define squareSize (/ (- 1000 (* gapSize numGaps)) numSquares))

(define baseColour (col/rgb r: [1.0 (scalar)] g: [0.0 (scalar)] b: [0.3 (scalar)] a: 1.0))
  (define backgroundColour (col/rgb r: 1.0 g: 1.0 b: 0.9))


(wash variation: 40
      lineWidth: 25
      lineSegments: 5
      colour: backgroundColour)

(loop (y from: 1 to: (- numSquares 1))
	(loop (x from: 1 to: (- numSquares 1))
          (let ((xPos (mapToPosition at: x))
                (yPos (mapToPosition at: y))
                (distanceFromCentre (math/distance2D aX: [500 (inRange min:0 max: 1000)]
                                                     aY: [800 (inRange min:0 max: 1000)]
                                                     bX: xPos
                                                     bY: yPos))
                (volatility (math/clamp val: (/ (- 500 distanceFromCentre)
                                                [12 (inRange min:5 max: 50)])
                                        min: 0
                                        max: 50)))
            (bezier-stroked-rect x: xPos
                                 y: yPos
                                 colourVolatility: [20 (inRange min: 2 max: 40)]
                                 volatility: volatility
                                 seed: (+ x (* y numSquares))
                                 width: squareSize
                                 height: squareSize
                                 colour: baseColour))))

(define (mapToPosition at: 0)
  (+ (* (+ gapSize squareSize) at) (/ squareSize 2) gapSize))



(define (bezier-stroked-rect x: 0
                             y: 0
                             width: 10
                             height: 10
                             colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 a: 0.5)
                             colourVolatility: 0
                             volatility: 0
                             overlap: 3
                             iterations: 10
                             seed: "shabba")
  (let ((thWidth (/ width 3))
        (thHeight (/ height 3))
        (vol volatility)

        (startX (- x (/ width 2)))
        (startY (- y (/ height 2)))

        (hDelta (/ height iterations))
        (hStripWidth (/ height iterations))
        (halfHStripWidth (/ hStripWidth 2))

        (vDelta (/ width iterations))
        (vStripWidth (/ width iterations))
        (halfVStripWidth (/ vStripWidth 2))

        (rng (rng/signed seed: seed))

        (halfAlpha (/ (col/getAlpha colour: colour) 2))
        (labColour (col/setAlpha colour: (col/convert format: LAB colour: colour)
                                 alpha: halfAlpha)))
                                        ; horizontal strips
    (loop (i to: iterations)
          (let (((rx1 ry1 rx2 ry2 rx3 ry3 rx4 ry4) (take num: 8 from: rng))
                (lightness (+ (col/getLightness colour: labColour)
                              (* colourVolatility (take from: rng))))
                (current-colour (col/setLightness colour: labColour l: lightness)))
            (bezier tessellation: 10
                    lineWidth: (+ overlap hStripWidth)
                    coords: (pair
                             (+ (+ (* rx1 vol) startX)
                                (* 0 thWidth))
                             (+ (+ (* i hDelta) (* ry1 vol) startY)
                                halfHStripWidth)

                             (+ (+ (* rx2 vol) startX)
                                (* 1 thWidth))
                             (+ (+ (* i hDelta) (* ry2 vol) startY)
                                halfHStripWidth)

                             (+ (+ (* rx3 vol) startX)
                                (* 2 thWidth))
                             (+ (+ (* i hDelta) (* ry3 vol) startY)
                                halfHStripWidth)

                             (+ (+ (* rx4 vol) startX)
                                (* 3 thWidth))
                             (+ (+ (* i hDelta) (* ry4 vol) startY)
                                halfHStripWidth))
                    colour: current-colour)))
                                        ; vertical strips
    (loop (i to: iterations)
          (let (((rx1 ry1 rx2 ry2 rx3 ry3 rx4 ry4) (take num: 8 from: rng))
                (lightness (+ (col/getLightness colour: labColour)
                              (* colourVolatility (take from: rng))))
                (current-colour (col/setLightness colour: labColour l: lightness)))
            (bezier tessellation: 10
                    lineWidth: (+ overlap vStripWidth)
                    coords: (pair
                             (+ (+ (* i vDelta) (* rx1 vol) startX)
                                halfVStripWidth)
                             (+ (+ (* ry1 vol) startY)
                                (* 0 thHeight))

                             (+ (+ (* i vDelta) (* rx2 vol) startX)
                                halfVStripWidth)
                             (+ (+ (* ry2 vol) startY)
                                (* 1 thHeight))

                             (+ (+ (* i vDelta) (* rx3 vol) startX)
                                halfVStripWidth)
                             (+ (+ (* ry3 vol) startY)
                                (* 2 thHeight))

                             (+ (+ (* i vDelta) (* rx4 vol) startX)
                                halfVStripWidth)
                             (+ (+ (* ry4 vol) startY)
                                (* 3 thHeight)))
                    colour: current-colour)))))


(define (accumulated-rect x: 0
                          y: 0
                          width: 10
                          height: 10
                          colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 a: 0.5)
                          volatility: 0
                          passes: 1
                          seed: "asdf")
  (let ((halfWidth (/ width 2))
        (halfHeight (/ height 2))
        (alpha (col/getAlpha colour: colour))
        (passColour (col/setAlpha colour: colour alpha: (/ alpha passes)))
        (rng (rng/signed seed: seed)))
    (onMatrixStack
     (translate x: x y: y)
     (loop (i to: passes)
           (let (((rr xr yr) (take num: 3 from: rng)))
             (onMatrixStack
              (rotate angle: (* rr 0.02 volatility))
              (rect x: (* xr 5 volatility)
                    y: (* yr 5 volatility)
                    width: width
                    height: height
                    colour: passColour)))))))

(define (v x: 0 y: 0 z: 0 scale: 1)
  (+ y (* scale (perlin/unsigned x: x y: y z: z))))

(define (wash variation: 200
              lineWidth: 70
              lineSegments: 5
              colour: (col/rgb r: 0.627 g: 0.627 b: 0.627 a: 0.4)
              seed: 272)
  (loop (h from: -20 to: 1020 step: 20)
        (bezier tessellation: lineSegments
                lineWidth: lineWidth
                coords: (pair
                         0 (v x: 0.10 y: h z: seed scale: variation)
                         333 (v x: 333.33 y: h z: seed scale: variation)
                         666 (v x: 666.66 y: h z: seed scale: variation)
                         1000 (v x: 1000.10 y: h z: seed scale: variation))
                colour: colour)
        (bezier tessellation: lineSegments
                lineWidth: lineWidth
                coords: (pair
                         (v x: 0.10 y: h z: seed scale: variation) 0
                         (v x: 333.33 y: h z: seed scale: variation) 333
                         (v x: 666.66 y: h z: seed scale: variation) 666
                         (v x: 1000.10 y: h z: seed scale: variation) 1000)
                colour: colour)))`;
  return code;
}


// execute the function and log the time that it takes
function withTiming(fn) {
  const before = new Date();
  fn();
  const after = new Date();
  const duration = after - before;
  console.log('Time: ' + duration + 'ms');
}

function setupUI(renderer) {
  const d = document;

  const textArea = d.getElementById('codemirror-textarea');

  textArea.value = initialCode();

  const editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: false,
    mode: 'scheme',
    autoCloseBrackets: true,
    matchBrackets: true,
    extraKeys: {
      'Ctrl-E': function() {
        withTiming(() => renderScript(renderer, editor.getValue()));
        return false;
      }
    }});
}

function copyRenderCanvasToImageElement(renderer, parent) {

  const newElement = document.createElement('img');
  parent.appendChild(newElement);

  newElement.width = 250;
  newElement.height = 250;
  newElement.src = renderer.getImageData();
  newElement.className += 'phenotype';

}

function setupSelectorUI(renderer, form) {
  const gallery = document.getElementById('gallery-container');

  let env = Runtime.createEnv();
  env = Bind.addBindings(env, renderer);
  const ast = Runtime.buildAst(env, form);
  const traits = Genetic.buildTraits(ast);
  let genotype = Genetic.createGenotypeFromInitialValues(traits);

  renderer.preDrawScene();
  Runtime.evalAst(env, ast, genotype);
  renderer.postDrawScene();
  copyRenderCanvasToImageElement(renderer, gallery);


  let i = 0;
  setTimeout(function go() {
    if(i < 100) {
      genotype = Genetic.createGenotypeFromTraits(traits, i+322);

      renderer.preDrawScene();
      Runtime.evalAst(env, ast, genotype);
      renderer.postDrawScene();
      copyRenderCanvasToImageElement(renderer, gallery);

      i++;
      setTimeout(go);
    }
  });
}

const SeniWebApplication = {
  mainFn() {
    const renderer = new Renderer('render-canvas');
    setupUI(renderer);
    renderScript(renderer, initialCode());
  },

  selectorMainFn() {
    const renderer = new Renderer('render-canvas');

    setupSelectorUI(renderer, initialCode());

    console.log(Trivia.getTitle());
  }
};

export default SeniWebApplication;
