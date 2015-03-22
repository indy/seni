import Renderer from './seni/Renderer';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';
import Bind from './seni/Bind';
import Trivia from './seni/Trivia';


const SeniMode = {
  authoring: 0,
  selecting: 1
};

let gCurrentMode = SeniMode.authoring;
let gRenderer;
let gEditor;

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
  const code = `(wash colour: (col/rgb r: 0.827 g: 0.827 b: 0.827 a: 0.4)
      seed: 88)

(define baseColour (col/rgb r: 0.9 g: 0.0 b: 0.0 a: 1.0))
(define border 30)

(define squareSize (/ (- 1000 (* 3 border)) 2))
(define squareRadius (/ squareSize 2))
(define squarePosMin (+ border squareRadius))
(define squarePosMax (- 1000 (+ border squareRadius)))

(rect x: squarePosMin y: squarePosMax
      width: squareSize height: squareSize
      colour: baseColour)

(accumulated-rect x: squarePosMax y: squarePosMax
                  width: squareSize height: squareSize
                  colour: baseColour
                  volatility: 1.5
                  passes: 50)

(bezier-stroked-rect x: squarePosMin y: squarePosMin
                     width: squareSize height: squareSize
                     colour: baseColour
                     volatility: 10.0
                     overlap: 3.0)

(bezier-stroked-rect x: squarePosMax y: squarePosMin
                     width: squareSize height: squareSize
                     colour: baseColour
                     colourVolatility: 10
                     volatility: 10.0
                     overlap: 3.0
                     iterations: 79)



(define (accumulated-rect x: 0
                          y: 0
                          width: 10
                          height: 10
                          colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 a: 0.5)
                          volatility: 0
                          passes: 1)
  (let ((halfWidth (/ width 2))
        (halfHeight (/ height 2))
        (alpha (col/getAlpha colour: colour))
        (passColour (col/setAlpha colour: colour alpha: (/ alpha passes)))
        (rng (rng/signed seed: "asdf")))
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
                colour: colour)))
`;
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

  gEditor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: false,
    mode: 'scheme',
    autoCloseBrackets: true,
    matchBrackets: true,
    extraKeys: {
      'Ctrl-E': function() {
        withTiming(() => renderScript(renderer, gEditor.getValue()));
        return false;
      },
      'Ctrl-D': function() {
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

  gallery.innerHTML = '';

  renderer.preDrawScene();
  Runtime.evalAst(env, ast, genotype);
  renderer.postDrawScene();
  copyRenderCanvasToImageElement(renderer, gallery);


  let i = 0;

  // a quick hack to get a pseudo-random string
  let random = new Date();
  random = random.toGMTString();
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to authoring mode
    if(i < 25 && gCurrentMode === SeniMode.selecting) {

      genotype = Genetic.createGenotypeFromTraits(traits, i+random);

      renderer.preDrawScene();
      Runtime.evalAst(env, ast, genotype);
      renderer.postDrawScene();
      copyRenderCanvasToImageElement(renderer, gallery);

      i++;
      setTimeout(go);
    }
  });
}

function switchMode(newMode) {
  console.log('switching mode to ' + newMode);
  gCurrentMode = newMode;

  const authorContainer = document.getElementById('author-container');
  const selectorContainer = document.getElementById('selector-container');

  const sourceCode = gEditor.getValue();

  if(gCurrentMode === SeniMode.authoring) {
    authorContainer.className = 'flex-container-h';
    selectorContainer.className = 'hidden';

    renderScript(gRenderer, sourceCode);
  } else {    // SeniMode.selecting
    authorContainer.className = 'hidden';
    selectorContainer.className = '';

    setupSelectorUI(gRenderer, sourceCode);
  }
}

const SeniWebApplication = {
  mainFn() {
    console.log(Trivia.getTitle());

    gCurrentMode = SeniMode.authoring;

    // Ctrl-D switches between author and selector mode
    document.addEventListener('keydown', function(event) {
      if(event.ctrlKey && event.keyCode === 68) {
        switchMode(1 - gCurrentMode);
        event.preventDefault();
      }
    }, false);

    gRenderer = new Renderer('render-canvas');
    setupUI(gRenderer);
    renderScript(gRenderer, initialCode());
  }
};

export default SeniWebApplication;
