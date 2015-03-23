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
  const code = `(bezierbezier)
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


    let onSwitchMode = function(event) {
        switchMode(1 - gCurrentMode);
        event.preventDefault();
    };

    // Ctrl-D switches between author and selector mode
    document.addEventListener('keydown', function(event) {
      if(event.ctrlKey && event.keyCode === 68) {
        onSwitchMode(event);
      }
    }, false);

    let selectorModeIcon = document.getElementById('selector-mode-icon');
    selectorModeIcon.addEventListener('click', onSwitchMode);

    let authorModeIcon = document.getElementById('author-mode-icon');
    authorModeIcon.addEventListener('click', onSwitchMode);

    gRenderer = new Renderer('render-canvas');
    setupUI(gRenderer);
    renderScript(gRenderer, initialCode());
  }
};

export default SeniWebApplication;
