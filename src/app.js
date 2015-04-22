/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import Renderer from './seni/Renderer';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';
import Bind from './seni/Bind';
import Trivia from './seni/Trivia';
import InitialCode from './InitialCode';

const SeniMode = {
  authoring: 0,
  selecting: 1
};

let gSeniApp = {
  currentMode: SeniMode.authoring,
  renderer: undefined,
  editor: undefined,
  containers: [],

  populationSize: 50,
  genotypes: [],

  form: undefined,
  ast: undefined,
  traits: undefined,
  env: undefined
};

/*
function debugEnv(env) {
  console.log(env);
  let keys = env.keySeq().toArray();
  // console.log(keys);
  let v;
  keys.forEach(k => {
    v = env.get(k);
    if(v.pb) {
      console.log(k, v.pb.doc);
    } else {
      console.log(k);
    }
  });
  //console.log(env.toJS());
}
*/

function renderScript(renderer, form) {
  let env = Runtime.createEnv();
  env = Bind.addBindings(env, renderer);

  //debugEnv(env);

  renderer.preDrawScene();
  const ast = Runtime.buildAst(env, form);

  const traits = Genetic.buildTraits(ast);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  Runtime.evalAst(env, ast, genotype);

  renderer.postDrawScene();
}

function initialCode() {
  return InitialCode.getCode();
}

// execute the function and log the time that it takes
function withTiming(msg, fn) {
  const before = new Date();
  fn();
  const after = new Date();
  const duration = after - before;
  console.log(msg, duration, 'ms');
}

function setupUI(renderer) {
  const d = document;

  const textArea = d.getElementById('codemirror-textarea');

  textArea.value = initialCode();

  gSeniApp.editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: false,
    mode: 'scheme',
    autoCloseBrackets: true,
    matchBrackets: true,
    extraKeys: {
      'Ctrl-E': function() {
        withTiming('renderTime',
                   () => renderScript(renderer, gSeniApp.editor.getValue()));
        return false;
      },
      'Ctrl-D': function() {
        return false;
      }
    }});
}

function createPhenotypeContainer(id) {
  const container = document.createElement('div');
  container.className = 'phenotype-container';

  container.id = 'pheno-' + id;

  const newElement = document.createElement('img');

  newElement.width = 250;
  newElement.height = 250;
  newElement.src = 'spinner.gif';
  newElement.className = 'phenotype';
  container.appendChild(newElement);

  /*
  const controls = document.createElement('div');
  controls.className = 'phenotype-control-container';
  controls.innerHTML = `
    <button class="phenotype-control-button">View</button>
    <input class="phenotype-control-checkbox"
                  type="checkbox" name="1" value="1">
  `;
  container.appendChild(controls);

  parent.appendChild(container);
   */

  return container;
}

function copyRenderCanvasIntoPhenotypeContainer(renderer, parent) {
  // get the image tag which has a class of phenotype
  let imageElement = parent.getElementsByClassName('phenotype')[0];
  imageElement.width = 250;
  imageElement.height = 250;
  imageElement.src = renderer.getImageData();
}

function renderPhenotypes(app) {

  let i = 0;
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to authoring mode
    if (i < app.containers.length &&
        app.currentMode === SeniMode.selecting) {

      let {phenotypeContainer} = app.containers[i];
      let genotype = app.genotypes[i];

      app.renderer.preDrawScene();
      Runtime.evalAst(app.env, app.ast, genotype);
      app.renderer.postDrawScene();
      copyRenderCanvasIntoPhenotypeContainer(app.renderer, phenotypeContainer);
      i++;
      setTimeout(go);
    }
  });
}

function setupSelectorUI(renderer, form) {
  const gallery = document.getElementById('gallery-container');

  gSeniApp.env = Bind.addBindings(Runtime.createEnv(), renderer);
  gSeniApp.ast = Runtime.buildAst(gSeniApp.env, form);
  gSeniApp.traits = Genetic.buildTraits(gSeniApp.ast);

  gallery.innerHTML = '';

  // create phenotype/genotype containers
  let i;
  let phenotypeContainer;
  gSeniApp.containers = [];
  for(i = 0; i < gSeniApp.populationSize; i++) {
    phenotypeContainer = createPhenotypeContainer(i);
    gallery.appendChild(phenotypeContainer);

    gSeniApp.containers.push({
      phenotypeContainer: phenotypeContainer,
      selected: false
    });
  }

  // add genotypes to the containers
  let genotype;
  let random = new Date();
  random = random.toGMTString();
  for(i = 0; i < gSeniApp.populationSize; i++) {
    if(i === 0) {
      genotype = Genetic.createGenotypeFromInitialValues(gSeniApp.traits);
    } else {
      genotype = Genetic.createGenotypeFromTraits(gSeniApp.traits, i + random);
    }
    gSeniApp.genotypes[i] = genotype;
  }

  // render the phenotypes
  renderPhenotypes(gSeniApp);
}

function switchMode(newMode) {
  console.log('switching mode to ' + newMode);
  gSeniApp.currentMode = newMode;

  const authorContainer = document.getElementById('author-container');
  const selectorContainer = document.getElementById('selector-container');

  const sourceCode = gSeniApp.editor.getValue();

  if (gSeniApp.currentMode === SeniMode.authoring) {
    authorContainer.className = 'flex-container-h';
    selectorContainer.className = 'hidden';

    renderScript(gSeniApp.renderer, sourceCode);
  } else {    // SeniMode.selecting
    authorContainer.className = 'hidden';
    selectorContainer.className = '';

    setupSelectorUI(gSeniApp.renderer, sourceCode);
  }
}

function toggleSelection(element) {
  let m = element.id.match(/pheno-(\d+)/);
  if(m.length === 2) {
    let index = Number.parseInt(m[1], 10);
    element.classList.toggle('selected');
    gSeniApp.containers[index].selected = !gSeniApp.containers[index].selected;
  } else {
    console.log('unexpected element id for phenotype container: ', element.id);
  }
}

function onNextGen(event) {
  console.log('onNextGen');
  event.preventDefault();

  // get the selected genotypes for the next generation
  let chosen = [];
  for(let i = 0; i < gSeniApp.populationSize; i++) {
    if(gSeniApp.containers[i].selected === true) {
      chosen.push(gSeniApp.genotypes[i]);
    }
  }

  gSeniApp.genotypes = Genetic.nextGeneration(chosen, gSeniApp.populationSize);

  // render the genotypes
  renderPhenotypes(gSeniApp);

  // clean up the dom and clear the selected state
  for(let i = 0; i < gSeniApp.populationSize; i++) {
    if(gSeniApp.containers[i].selected === true) {
      gSeniApp.containers[i].phenotypeContainer.classList.toggle('selected');
    }
    gSeniApp.containers[i].selected = false;
  }

}

const SeniWebApplication = {
  mainFn() {
    console.log(Trivia.getTitle());

    gSeniApp.currentMode = SeniMode.authoring;

    let onSwitchMode = function(event) {
      switchMode(1 - gSeniApp.currentMode);
      event.preventDefault();
    };

    // Ctrl-D switches between author and selector mode
    document.addEventListener('keydown', function(event) {
      if (event.ctrlKey && event.keyCode === 68) {
        onSwitchMode(event);
      }
    }, false);

    let selectorModeIcon = document.getElementById('selector-mode-icon');
    selectorModeIcon.addEventListener('click', onSwitchMode);

    let authorModeIcon = document.getElementById('author-mode-icon');
    authorModeIcon.addEventListener('click', onSwitchMode);

    gSeniApp.renderer = new Renderer('render-canvas');
    setupUI(gSeniApp.renderer);
    renderScript(gSeniApp.renderer, initialCode());

    let galleryContainer = document.getElementById('gallery-container');
    galleryContainer.addEventListener('click', function(event) {
      let phenoContainer = event.target.parentNode;
      toggleSelection(phenoContainer);
    });

    let nextGenIcon = document.getElementById('next-gen-icon');
    nextGenIcon.addEventListener('click', onNextGen);
  }
};

export default SeniWebApplication;
