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

function renderScriptToImage(seniApp, ast, genotype, imageElement) {

  const renderer = seniApp.renderer;

  renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);

  Runtime.evalAst(seniApp.env, ast, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
}

function renderScript(seniApp, form) {
  seniApp.form = form;
  const imageElement = document.getElementById('render-img');

  const ast = Runtime.buildAst(seniApp.env, form);

  const traits = Genetic.buildTraits(ast);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  renderScriptToImage(seniApp, ast, genotype, imageElement);
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

function addClickEvent(id, fn) {
  const element = document.getElementById(id);
  element.addEventListener('click', fn);
}

// when user has clicked on a phenotype in the selector UI,
// traverse up the card until we get to a dom element that
// contains the phenotype's index number in it's id
function getPhenoIdFromDom(element) {
  while(element) {
    const m = element.id.match(/pheno-(\d+)/);
    if(m && m.length === 2) {
      const index = Number.parseInt(m[1], 10);
      return [index, element];
    } else {
      element = element.parentNode;
    }
  }
  return [-1, null];
}

function renderHighRes(seniApp, element) {
  const [index, _] = getPhenoIdFromDom(element);
  if(index !== -1) {
    let genotype = seniApp.genotypes[index];
    const imageElement = document.getElementById('high-res-image');
    imageElement.classList.toggle('hidden');
    const ast = Runtime.buildAst(seniApp.env, seniApp.form);
    renderScriptToImage(seniApp, ast, genotype, imageElement);
  }
}

function toggleSelection(seniApp, element) {
  const [index, e] = getPhenoIdFromDom(element);
  if(index !== -1) {
    const cardImage = e.getElementsByClassName('card-image')[0];
    cardImage.classList.toggle('selected');

    const c = seniApp.containers[index];
    c.selected = !c.selected;
  }
}

function renderPhenotypes(app) {

  let i = 0;
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to authoring mode
    if (i < app.containers.length && app.currentMode === SeniMode.selecting) {

      const genotype = app.genotypes[i];

      const imageElement = app.containers[i].imageElement;

      app.renderer.preDrawScene(imageElement.clientWidth,
                                imageElement.clientHeight);

      Runtime.evalAst(app.env, app.ast, genotype);
      app.renderer.postDrawScene();

      imageElement.src = app.renderer.getImageData();

      i++;
      setTimeout(go);
    }
  });
}

function showPlaceholderImages(seniApp) {
  for(let i = 0; i < seniApp.populationSize; i++) {
    const imageElement = seniApp.containers[i].imageElement;
    imageElement.src = seniApp.placeholder;
  }
}

function onNextGen(seniApp) {
  showPlaceholderImages(seniApp);
  // get the selected genotypes for the next generation
  let chosen = [];
  for(let i = 0; i < seniApp.populationSize; i++) {
    if(seniApp.containers[i].selected === true) {
      chosen.push(seniApp.genotypes[i]);
    }
  }

  seniApp.genotypes = Genetic.nextGeneration(chosen, seniApp.populationSize);

  // render the genotypes
  renderPhenotypes(seniApp);

  // clean up the dom and clear the selected state
  for(let i = 0; i < seniApp.populationSize; i++) {
    if(seniApp.containers[i].selected === true) {
      const element = seniApp.containers[i].phenotypeContainer;
      const cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.toggle('selected');
    }
    seniApp.containers[i].selected = false;
  }
}

function createPhenotypeContainer(id, placeholderImage) {
  const container = document.createElement('div');

  container.className = 'phenotype-container col s6 m4 l3';
  container.id = 'pheno-' + id;

  container.innerHTML = `
    <div class="card">
      <div class="card-image">
        <img class="phenotype" src="${placeholderImage}">
      </div>
      <div class="card-action">
        <a href="#" class="render">Render</a>
      </div>
    </div>
    `;

  return container;
}

function setupUI(seniApp) {
  const d = document;

  const textArea = d.getElementById('codemirror-textarea');

  textArea.value = initialCode();

  seniApp.editor = CodeMirror.fromTextArea(textArea, {
    lineNumbers: false,
    mode: 'scheme',
    autoCloseBrackets: true,
    matchBrackets: true,
    extraKeys: {
      'Ctrl-E': function() {
        const source = seniApp.editor.getValue();
        withTiming('renderTime', () =>
                   renderScript(seniApp, source));
        return false;
      },
      'Ctrl-D': function() {
        return false;
      }
    }});

  addClickEvent('selector-mode-icon', event => {
    if(seniApp.currentMode !== SeniMode.selecting) {
      switchMode(seniApp, SeniMode.selecting);
    }
    event.preventDefault();
  });

  addClickEvent('author-mode-icon', event => {
    if(seniApp.currentMode !== SeniMode.authoring) {
      switchMode(seniApp, SeniMode.authoring);
    }
    event.preventDefault();
  });




  addClickEvent('action-eval', () => {
    let source = seniApp.editor.getValue();
    withTiming('renderTime', () => renderScript(seniApp, source));
  });

  addClickEvent('gallery-container', event => {
    let target = event.target;
    if(target.classList.contains('render')) {
      renderHighRes(seniApp, target);
    } else {
      toggleSelection(seniApp, target);
    }
    event.preventDefault();
  });

  addClickEvent('action-next-gen', () => {
    onNextGen(seniApp);
  });

  addClickEvent('high-res-image', (event) => {
    event.target.classList.toggle('hidden');
  });


  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    if (event.ctrlKey && event.keyCode === dKey &&
        seniApp.currentMode === SeniMode.selecting) {
      event.preventDefault();
      onNextGen(seniApp);
    }
  }, false);

  const gallery = document.getElementById('gallery-container');
  gallery.innerHTML = '';

  let i;
  let phenotypeContainer;
  seniApp.containers = [];

  let row;

  row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

  for(i = 0; i < seniApp.populationSize; i++) {
    phenotypeContainer = createPhenotypeContainer(i, seniApp.placeholder);
    row.appendChild(phenotypeContainer);

    const imageElement =
            phenotypeContainer.getElementsByClassName('phenotype')[0];

    seniApp.containers.push({
      phenotypeContainer,
      imageElement,
      selected: false
    });
  }
}

function setupSelectorUI(seniApp, form) {

  showPlaceholderImages(seniApp);

  const renderer = seniApp.renderer;
  const gallery = document.getElementById('gallery-container');

  seniApp.ast = Runtime.buildAst(seniApp.env, form);
  seniApp.traits = Genetic.buildTraits(seniApp.ast);

  // create phenotype/genotype containers
  let i;

  // add genotypes to the containers
  let genotype;
  let random = new Date();
  random = random.toGMTString();
  for(i = 0; i < seniApp.populationSize; i++) {
    if(i === 0) {
      genotype = Genetic.createGenotypeFromInitialValues(seniApp.traits);
    } else {
      genotype = Genetic.createGenotypeFromTraits(seniApp.traits, i + random);
    }
    seniApp.genotypes[i] = genotype;
  }

  // render the phenotypes
  renderPhenotypes(seniApp);
}

function switchMode(seniApp, newMode) {
  console.log('switching mode to ' + newMode);
  seniApp.currentMode = newMode;

  const authorContainer = document.getElementById('author-container');
  const selectorContainer = document.getElementById('selector-container');

  const sourceCode = seniApp.editor.getValue();

  if (seniApp.currentMode === SeniMode.authoring) {
    authorContainer.className = 'flex-container-h';
    selectorContainer.className = 'hidden';

    renderScript(seniApp, sourceCode);
  } else {    // SeniMode.selecting
    authorContainer.className = 'hidden';
    selectorContainer.className = '';

    setupSelectorUI(seniApp, sourceCode);
  }
}

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  const ac = document.getElementById('author-container');
  ac.style.height = (ac.offsetHeight - navbar.offsetHeight) + 'px';

  const sc = document.getElementById('selector-container');
  sc.style.height = (sc.offsetHeight - navbar.offsetHeight) + 'px';
}

function polluteGlobalDocument(seniApp) {
  document.seni = {};
  document.seni.title = Trivia.getTitle;
  document.seni.help = function(name) {
    const v = seniApp.env.get(name);
    if(v.pb) {
      const binding = v.pb;       // publicBinding
      const args = JSON.stringify(binding.defaults, null, ' ');
      console.log(name + ':', binding.doc);
      console.log('default arguments', args);
    }
  };
}

const SeniWebApplication = {
  mainFn() {
    const seniApp = {
      currentMode: SeniMode.authoring,
      renderer: undefined,
      editor: undefined,
      containers: [],

      placeholder: 'spinner.gif',
      populationSize: 24,
      genotypes: [],

      form: undefined,
      ast: undefined,
      traits: undefined,
      env: undefined
    };

    resizeContainers();

    seniApp.renderer = new Renderer('render-canvas');
    seniApp.env = Bind.addBindings(Runtime.createEnv(), seniApp.renderer);
    seniApp.currentMode = SeniMode.authoring;

    polluteGlobalDocument(seniApp);

    setupUI(seniApp);
    renderScript(seniApp, initialCode());
  }
};

export default SeniWebApplication;
