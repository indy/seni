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

function renderScript(seniApp, form) {

  const renderer = seniApp.renderer;
  const imageElement = document.getElementById('render-img');

  renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);
  const ast = Runtime.buildAst(seniApp.env, form);

  const traits = Genetic.buildTraits(ast);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  Runtime.evalAst(seniApp.env, ast, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
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

function renderPhenotypes(app) {

  let i = 0;
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to authoring mode
    if (i < app.containers.length &&
        app.currentMode === SeniMode.selecting) {

      const {phenotypeContainer} = app.containers[i];
      const genotype = app.genotypes[i];

      const imageElement =
              phenotypeContainer.getElementsByClassName('phenotype')[0];

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

function setupSelectorUI(seniApp, form) {
  const renderer = seniApp.renderer;
  const gallery = document.getElementById('gallery-container');

  seniApp.ast = Runtime.buildAst(seniApp.env, form);
  seniApp.traits = Genetic.buildTraits(seniApp.ast);

  gallery.innerHTML = '';

  // create phenotype/genotype containers
  let i;
  let phenotypeContainer;
  seniApp.containers = [];

  let row;

  row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

  for(i = 0; i < seniApp.populationSize; i++) {
    phenotypeContainer = createPhenotypeContainer(i);
    row.appendChild(phenotypeContainer);

    seniApp.containers.push({
      phenotypeContainer: phenotypeContainer,
      selected: false
    });
  }

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

  // todo: find better way of managing UI state than these horrible toggle calls
  const liAuthorMode = document.getElementById('li-author-mode');
  liAuthorMode.classList.toggle('active');
  const liSelectorMode = document.getElementById('li-selector-mode');
  liSelectorMode.classList.toggle('active');
}

function toggleSelection(seniApp, element) {
  while(element) {
    const m = element.id.match(/pheno-(\d+)/);
    if(m && m.length === 2) {
      const index = Number.parseInt(m[1], 10);

      const cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.toggle('selected');

      const c = seniApp.containers[index];
      c.selected = !c.selected;

      return;
    } else {
      element = element.parentNode;
    }
  }
}

function onNextGen(seniApp) {
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

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  const ac = document.getElementById('author-container');
  ac.style.height = (ac.offsetHeight - navbar.offsetHeight) + 'px';

  const sc = document.getElementById('selector-container');
  sc.style.height = (sc.offsetHeight - navbar.offsetHeight) + 'px';
}

function addClickEvent(id, fn) {
  const element = document.getElementById(id);
  element.addEventListener('click', fn);
}

function polluteGlobalNamespace(seniApp) {
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

    polluteGlobalNamespace(seniApp);

    seniApp.currentMode = SeniMode.authoring;

    setupUI(seniApp);
    renderScript(seniApp, initialCode());

    const onSwitchMode = function(event) {
      switchMode(seniApp, 1 - seniApp.currentMode);
      event.preventDefault();
    };

    addClickEvent('selector-mode-icon', onSwitchMode);
    addClickEvent('author-mode-icon', onSwitchMode);

    addClickEvent('action-eval', () => {
      let source = seniApp.editor.getValue();
      withTiming('renderTime', () => renderScript(seniApp, source));
    });

    addClickEvent('gallery-container', event => {
      toggleSelection(seniApp, event.target);
    });

    addClickEvent('action-next-gen', () => {
      onNextGen(seniApp);
    });

    // Ctrl-D renders the next generation
    document.addEventListener('keydown', event => {
      if (event.ctrlKey && event.keyCode === 68 &&
          seniApp.currentMode === SeniMode.selecting) {
        event.preventDefault();
        onNextGen(seniApp);
      }
    }, false);
  }
};

export default SeniWebApplication;
