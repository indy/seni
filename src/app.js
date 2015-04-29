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

/* eslint-disable no-unused-vars */

const SeniMode = {
  authoring: 0,
  selecting: 1
};

let gSeniApp = {
  currentMode: SeniMode.authoring,
  renderer: undefined,
  editor: undefined,
  containers: [],

  populationSize: 48,
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
        let source = gSeniApp.editor.getValue();
        withTiming('renderTime', () =>
                   renderScript(renderer, source));
        return false;
      },
      'Ctrl-D': function() {
        return false;
      }
    }});
}

function createPhenotypeContainer(id) {
  const container = document.createElement('div');

  container.className = 'phenotype-container col s6 m4 l3';
  container.id = 'pheno-' + id;

  container.innerHTML = `
    <div class="card">
      <div class="card-image">
        <img class="phenotype" src="spinner.gif">
      </div>
      <div class="card-action">
        <a href="#">Preview</a>
      </div>
    </div>
    `;

  return container;
}

function copyRenderCanvasIntoPhenotypeContainer(renderer, parent) {
  // get the image tag which has a class of phenotype
  let imageElement = parent.getElementsByClassName('phenotype')[0];
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

  let row;

  row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

  for(i = 0; i < gSeniApp.populationSize; i++) {
    phenotypeContainer = createPhenotypeContainer(i);
    row.appendChild(phenotypeContainer);

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

  // todo: find better way of managing UI state than these horrible toggle calls
  let liAuthorMode = document.getElementById('li-author-mode');
  liAuthorMode.classList.toggle('active');
  let liSelectorMode = document.getElementById('li-selector-mode');
  liSelectorMode.classList.toggle('active');
}

function toggleSelection(element) {
  while(element) {
    let m = element.id.match(/pheno-(\d+)/);
    if(m && m.length === 2) {
      let index = Number.parseInt(m[1], 10);

      let cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.toggle('selected');

      let c = gSeniApp.containers[index];
      c.selected = !c.selected;

      return;
    } else {
      element = element.parentNode;
    }
  }
}

function onNextGen() {
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
      let element = gSeniApp.containers[i].phenotypeContainer;
      let cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.toggle('selected');
    }
    gSeniApp.containers[i].selected = false;
  }
}

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  let ac = document.getElementById('author-container');
  ac.style.height = (ac.offsetHeight - navbar.offsetHeight) + 'px';

  let sc = document.getElementById('selector-container');
  sc.style.height = (sc.offsetHeight - navbar.offsetHeight) + 'px';
}

const SeniWebApplication = {
  mainFn() {

    resizeContainers();

    console.log(Trivia.getTitle());

    gSeniApp.currentMode = SeniMode.authoring;

    let onSwitchMode = function(event) {
      switchMode(1 - gSeniApp.currentMode);
      event.preventDefault();
    };


    let selectorModeIcon = document.getElementById('selector-mode-icon');
    selectorModeIcon.addEventListener('click', onSwitchMode);

    let authorModeIcon = document.getElementById('author-mode-icon');
    authorModeIcon.addEventListener('click', onSwitchMode);

    gSeniApp.renderer = new Renderer('render-canvas');
    setupUI(gSeniApp.renderer);
    renderScript(gSeniApp.renderer, initialCode());


    let evalButton = document.getElementById('action-eval');
    evalButton.addEventListener('click', () => {
      let source = gSeniApp.editor.getValue();
      withTiming('renderTime', () =>
                 renderScript(gSeniApp.renderer, source));
    });

    let galleryContainer = document.getElementById('gallery-container');
    galleryContainer.addEventListener('click', event => {
      //let phenoContainer = event.target.parentNode;
      toggleSelection(event.target);
    });


    // Ctrl-D renders the next generation
    document.addEventListener('keydown', event => {
      if (event.ctrlKey &&
          event.keyCode === 68 &&
          gSeniApp.currentMode === SeniMode.selecting) {
        event.preventDefault();
        onNextGen();
      }
    }, false);

    let nextGenButton = document.getElementById('action-next-gen');
    nextGenButton.addEventListener('click', () => {
      onNextGen();
    });

  }
};

export default SeniWebApplication;
