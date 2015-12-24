/*
 *  Seni
 *  Copyright (C) 2015 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

import Renderer from './seni/Renderer';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';
import Bind from './seni/Bind';
import Trivia from './seni/Trivia';
import CodeMirrorConfig from './ui/CodeMirrorConfig';
import Util from './seni/Util';

const SeniMode = {
  gallery: 0,
  edit: 1,
  evolve: 2,
  numSeniModes: 3
};

function seniModeAsString(mode) {
  switch (mode) {
  case SeniMode.gallery:
    return 'SeniMode.gallery';
  case SeniMode.edit:
    return 'SeniMode.edit';
  case SeniMode.evolve:
    return 'SeniMode.evolve';
  default:
    return 'error unknown SeniMode value';
  }
}

function get(url) {
  return new Promise((resolve, reject) => {

    const req = new XMLHttpRequest();
    req.open('GET', url);

    req.onload = () => {
      // This is called even on 404 etc
      // so check the status
      if (req.status === 200) {
        // Resolve the promise with the response text
        resolve(req.response);
      }
      else {
        // Otherwise reject with the status text
        // which will hopefully be a meaningful error
        reject(Error(req.statusText));
      }
    };

    // Handle network errors
    req.onerror = () => {
      reject(Error('Network Error'));
    };

    // Make the request
    req.send();
  });
}

function getJSON(url) {
  return get(url).then(JSON.parse);
}

function getScriptFromEditor(seniApp) {
  seniApp.piece.script = seniApp.editor.getValue();
}

function ensureMode(seniApp, mode) {
  if (seniApp.currentMode === mode) {
    return;
  }
  seniApp.currentMode = mode;
  // todo: historyAdd(seniApp) ????
  updateUI(seniApp);
}

// function that takes a read-only seniApp and updates the UI
//
function updateUI(seniApp) {
  showCurrentMode(seniApp);

  switch (seniApp.currentMode) {
  case SeniMode.gallery :
    hideTopNavBar(seniApp);
    break;
  case SeniMode.edit :
    showTopNavBar(seniApp);
    showScriptInEditor(seniApp);
    timedRenderScript(seniApp, 'renderScript');
    break;
  case SeniMode.evolve :
    showTopNavBar(seniApp);
    getScriptFromEditor(seniApp);
    // if it's a change of mode into the SeniMode.evolve
    setupEvolveUI(seniApp);
    // else if there's been a change in selection ???
    break;
  }
}

// search the children of seniApp.navbar for elements with class 'klass'
// then add 'addClass' to them
function addNavbarClass(seniApp, klass, addClass) {
  const es = seniApp.navbar.getElementsByClassName(klass);
  for (let i = 0; i < es.length; i++) {
    es[i].classList.add(addClass);
  }
}

function removeNavbarClass(seniApp, klass, removeClass) {
  const es = seniApp.navbar.getElementsByClassName(klass);
  for (let i = 0; i < es.length; i++) {
    es[i].classList.remove(removeClass);
  }
}

function renderGenotypeToImage(seniApp, ast, genotype, imageElement, w, h) {

  const renderer = seniApp.renderer;

  if (w !== undefined && h !== undefined) {
    renderer.preDrawScene(w, h);
  } else {
    renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);
  }

  Runtime.evalAst(seniApp.env, ast, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
}

function renderScript(seniApp) {
  const imageElement = seniApp.renderImage;

  const script = seniApp.piece.script;
  const frontAst = Runtime.buildFrontAst(script);

  const backAst = Runtime.compileBackAst(frontAst);

  const traits = Genetic.buildTraits(backAst);

  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  // Runtime.logUnparse(frontAst, genotype);

  renderGenotypeToImage(seniApp, backAst, genotype, imageElement);
}

function timedRenderScript(seniApp, msg) {
  Util.withTiming(msg, () => renderScript(seniApp), false);
}

function addClickEvent(id, fn) {
  const element = document.getElementById(id);
  element.addEventListener('click', fn);
}

function addClickEventForClass(className, fn) {
  const elements = document.getElementsByClassName(className);
  // getElementsByClassName returns an array-like object
  for (let i = 0; i < elements.length; i++) {
    elements[i].addEventListener('click', fn);
  }
}

// when user has clicked on a phenotype in the evolve UI,
// traverse up the card until we get to a dom element that
// contains the phenotype's index number in it's id
function getPhenoIdFromDom(element) {
  while (element) {
    const m = element.id.match(/pheno-(\d+)/);
    if (m && m.length === 2) {
      const index = Number.parseInt(m[1], 10);
      return [index, element];
    } else {
      element = element.parentNode;
    }
  }
  return [-1, null];
}

function renderHighRes(seniApp, element) {
  /* eslint-disable no-unused-vars */
  const [index, _] = getPhenoIdFromDom(element);
  /* eslint-enable no-unused-vars */

  if (index !== -1) {
    const piece = seniApp.piece;
    const genotype = piece.genotypes[index];
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.remove('invisible');
    const frontAst = Runtime.buildFrontAst(piece.script);
    const backAst = Runtime.compileBackAst(frontAst);

    const imageElement = document.getElementById('high-res-image');
    const [width, height] = seniApp.highResolution;
    renderGenotypeToImage(seniApp, backAst, genotype, imageElement,
                          width, height);

    const holder = document.getElementById('holder');
    imageElement.style.height = `${holder.clientHeight}px`;
    imageElement.style.width = `${holder.clientWidth}px`;

    const linkElement = document.getElementById('high-res-link');
    linkElement.href = imageElement.src;
  }
}

function showEditFromEvolve(seniApp, element) {
  /* eslint-disable no-unused-vars */
  const [index, _] = getPhenoIdFromDom(element);
  /* eslint-enable no-unused-vars */

  if (index !== -1) {
    const piece = seniApp.piece;
    const genotype = piece.genotypes[index];
    const frontAst = seniApp.piece.frontAst;

    const script = Runtime.unparse(frontAst, genotype);

    seniApp.piece.script = script;

    ensureMode(seniApp, SeniMode.edit);
  }
}

function toggleSelection(seniApp, element) {
  const [index, e] = getPhenoIdFromDom(element);
  if (index !== -1) {
    const cardImage = e.getElementsByClassName('card-image')[0];
    cardImage.classList.toggle('selected');

    const c = seniApp.piece.phenotypes[index];
    c.selected = !c.selected;
  }
}

function renderPhenotypes(seniApp) {

  let i = 0;
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to edit mode
    const piece = seniApp.piece;
    if (i < piece.phenotypes.length &&
        seniApp.currentMode === SeniMode.evolve) {

      const genotype = piece.genotypes[i];
      const imageElement = piece.phenotypes[i].imageElement;

      renderGenotypeToImage(seniApp, piece.backAst, genotype, imageElement);

      i++;
      setTimeout(go);
    }
  });
}

function showPlaceholderImages(seniApp) {
  const piece = seniApp.piece;
  for (let i = 0; i < seniApp.populationSize; i++) {
    const imageElement = piece.phenotypes[i].imageElement;
    imageElement.src = seniApp.placeholder;
  }
}

function createInitialGenotypePopulation(piece, populationSize) {
  // add genotypes to the containers
  let genotype;
  let random = new Date();
  random = random.toGMTString();
  for (let i = 0; i < populationSize; i++) {
    if (i === 0) {
      genotype = Genetic.createGenotypeFromInitialValues(piece.traits);
    } else {
      genotype = Genetic.createGenotypeFromTraits(piece.traits, i + random);
    }
    piece.genotypes[i] = genotype;
  }
  return piece;
}

function genotypesFromSelectedPhenotypes(seniApp) {
  const piece = seniApp.piece;

  showPlaceholderImages(seniApp);

  if (piece.selectedGenotypes.length === 0) {
    // if this is the first generation and nothing has been selected
    // just randomize all of the phenotypes
    createInitialGenotypePopulation(piece, seniApp.populationSize);
  } else {
    piece.genotypes = Genetic.nextGeneration(piece.selectedGenotypes,
                                             seniApp.populationSize,
                                             seniApp.mutationRate,
                                             piece.traits);
  }
  historyAdd(seniApp);

  // render the genotypes
  renderPhenotypes(seniApp);

  // clean up the dom and clear the selected state
  for (let i = 0; i < seniApp.populationSize; i++) {
    if (piece.phenotypes[i].selected === true) {
      const element = piece.phenotypes[i].phenotypeElement;
      const cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.remove('selected');
    }
    piece.phenotypes[i].selected = false;
  }
}

function onNextGen(seniApp) {
  // get the selected genotypes for the next generation
  const piece = seniApp.piece;

  piece.selectedGenotypes = [];

  for (let i = 0; i < seniApp.populationSize; i++) {
    if (piece.phenotypes[i].selected === true) {
      piece.selectedGenotypes.push(piece.genotypes[i]);
    }
  }

  if (piece.selectedGenotypes.length === 0) {
    // no phenotypes were selected
    return;
  }

  historyAddSelectedGenotypes(piece.selectedGenotypes);
  genotypesFromSelectedPhenotypes(seniApp);
}

function createPhenotypeElement(id, placeholderImage) {
  const container = document.createElement('div');

  container.className = 'col s6 m4 l3';
  container.id = `pheno-${id}`;

  container.innerHTML = `
    <div class="card">
      <div class="card-image">
        <img class="phenotype" data-id="${id}" src="${placeholderImage}">
      </div>
      <div class="card-action">
        <a href="#" class="render">Render</a>
        <a href="#" class="edit">Edit</a>
      </div>
    </div>
    `;

  return container;
}

function setupEvolveUI(seniApp) {

//  getScriptFromEditor(seniApp);

  const allImagesLoadedSince = function(timeStamp) {
    const piece = seniApp.piece;
    for (let i = 0; i < seniApp.populationSize; i++) {
      if (piece.phenotypes[i].imageLoadTimeStamp < timeStamp) {
        return false;
      }
    }
    return true;
  };

  const initialTimeStamp = Date.now();

  showPlaceholderImages(seniApp);

  setTimeout(function go() {
    // wait until all of the placeholder load events have been received
    // otherwise there may be image sizing issues, especially with the
    // first img element
    if (allImagesLoadedSince(initialTimeStamp)) {
      const piece = seniApp.piece;

      piece.frontAst = Runtime.buildFrontAst(seniApp.piece.script);
      piece.backAst = Runtime.compileBackAst(piece.frontAst);
      piece.traits = Genetic.buildTraits(piece.backAst);

      createInitialGenotypePopulation(piece, seniApp.populationSize);

      // render the phenotypes
      renderPhenotypes(seniApp);

    } else {
      setTimeout(go, 20);
    }
  });
}

function hideTopNavBar(seniApp) {
  addNavbarClass(seniApp, 'to-gallery', 'hidden');
  addNavbarClass(seniApp, 'to-edit', 'hidden');
  addNavbarClass(seniApp, 'to-evolve', 'hidden');
}

function showTopNavBar(seniApp) {
  removeNavbarClass(seniApp, 'to-gallery', 'hidden');
  removeNavbarClass(seniApp, 'to-edit', 'hidden');
  removeNavbarClass(seniApp, 'to-evolve', 'hidden');
}


function showCurrentMode(seniApp) {
  // show the current container, hide the others
  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    seniApp.containers[i].className = i === seniApp.currentMode ? '' : 'hidden';
  }
}

/* eslint-disable no-unused-vars */
function showScriptInEditor(seniApp) {
  const editor = seniApp.editor;
  editor.getDoc().setValue(seniApp.piece.script);
  editor.refresh();
}

function showEditFromGallery(seniApp, element) {

  const getGalleryItemIdFromDom = function(e) {
    while (e) {
      const m = e.id.match(/gallery-item-(\d+)/);
      if (m && m.length === 2) {
        const idx = Number.parseInt(m[1], 10);
        return [idx, e];
      } else {
        e = e.parentNode;
      }
    }
    return [-1, null];
  };

  const [index, _] = getGalleryItemIdFromDom(element);
  if (index !== -1) {
    const url = `/gallery/${index}`;

    get(url).catch(() => {
      console.error(`cannot connect to ${url}`);
    }).then(data => {
      // todo: construct a new piece object
      seniApp.piece.script = data;

      ensureMode(seniApp, SeniMode.edit);
    });
  }
}
/* eslint-enable no-unused-vars */

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  const edit = document.getElementById('edit-container');
  edit.style.height = `${window.innerHeight - navbar.offsetHeight}px`;

  const evolve = document.getElementById('evolve-container');
  evolve.style.height = `${window.innerHeight - navbar.offsetHeight}px`;
}

function polluteGlobalDocument(seniApp) {
  document.seni = {};
  document.seni.title = Trivia.getTitle;
  document.seni.help = function(name, showDefaultArgs = false) {
    const v = seniApp.env.get(name);
    if (v.pb) {
      const binding = v.pb;       // publicBinding
      console.log(`${name}: ${binding.doc}`);

      if (showDefaultArgs) {
        const args = JSON.stringify(binding.defaults, null, ' ');
        console.log('default arguments', args);
      }
    }
  };

  document.seni.ls = function() {
    const env = seniApp.env;
    const keys = env.keys();

    const res = [];
    for (let k = keys.next(); k.done === false; k = keys.next()) {
      res.push(k.value);
    }
    res.sort();
    res.map(name => console.log(name));
  };
  document.seni.seniApp = seniApp;
}

function setupUI(seniApp) {
  const d = document;

  seniApp.navbar = document.getElementById('seni-navbar');
  seniApp.renderImage = document.getElementById('render-img');
  seniApp.containers = [
    document.getElementById('gallery-container'),
    document.getElementById('edit-container'),
    document.getElementById('evolve-container')
  ];

  // hide the navbar links because we start off in gallery mode
  addNavbarClass(seniApp, 'to-gallery', 'hidden');
  addNavbarClass(seniApp, 'to-edit', 'hidden');
  addNavbarClass(seniApp, 'to-evolve', 'hidden');

  const blockIndent = function(editor, from, to) {
    editor.operation(() => {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  const codeMirror = CodeMirrorConfig.defineSeniMode();
  const config = CodeMirrorConfig.defaultConfig;
  config.extraKeys = {
    'Ctrl-E': () => {
      getScriptFromEditor(seniApp);
      timedRenderScript(seniApp, 'renderScript');
      return false;
    },
    'Ctrl-D': () => false,
    'Ctrl-I': () => {
      const numLines = seniApp.editor.doc.size;
      blockIndent(seniApp.editor, 0, numLines);
      console.log('indenting', numLines, 'lines');
      return false;
    }
  };

  const textArea = d.getElementById('codemirror-textarea');
  seniApp.editor = codeMirror.fromTextArea(textArea, config);

  const galleryModeHandler = event => {
    ensureMode(seniApp, SeniMode.gallery);
    event.preventDefault();
  };

  const evolveModeHandler = event => {
    ensureMode(seniApp, SeniMode.evolve);
    event.preventDefault();
  };

  const editModeHandler = event => {
    ensureMode(seniApp, SeniMode.edit);
    event.preventDefault();
  };


  addClickEvent('evolve-mode-icon', evolveModeHandler);
  addClickEventForClass('to-evolve', evolveModeHandler);

  addClickEvent('edit-mode-icon', editModeHandler);
  addClickEventForClass('to-edit', editModeHandler);

  addClickEventForClass('to-gallery', galleryModeHandler);

  addClickEvent('shuffle-icon', event => {
    genotypesFromSelectedPhenotypes(seniApp);
    event.preventDefault();
  });

  addClickEvent('action-eval', () => {
    seniApp.piece.script = seniApp.editor.getValue();
    timedRenderScript(seniApp, 'renderScript');
  });

  addClickEvent('action-add', () => {
    seniApp.piece.script = '';
    ensureMode(seniApp, SeniMode.edit);
  });

  addClickEvent('gallery-list', event => {
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      showEditFromGallery(seniApp, target);
    }
    event.preventDefault();
  });

  addClickEvent('phenotype-gallery', event => {
    const target = event.target;
    if (target.classList.contains('render')) {
      renderHighRes(seniApp, target);
    } else if (target.classList.contains('edit')) {
      showEditFromEvolve(seniApp, target);
    } else {
      toggleSelection(seniApp, target);
    }
    event.preventDefault();
  });

  addClickEvent('action-next-gen', () => {
    onNextGen(seniApp);
  });

  addClickEvent('high-res-close', event => {
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.add('invisible');

    event.preventDefault();
  });

  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    if (event.ctrlKey && event.keyCode === dKey &&
        seniApp.currentMode === SeniMode.evolve) {
      event.preventDefault();
      onNextGen(seniApp);
    }
  }, false);

  const piece = seniApp.piece;

  // invoked on every load event for an img tag
  const imageLoadHandler = event => {
    const imageId = event.target.getAttribute('data-id');
    piece.phenotypes[imageId].imageLoadTimeStamp = event.timeStamp;
  };

  const gallery = document.getElementById('phenotype-gallery');
  gallery.innerHTML = '';

  let phenotypeElement, imageElement;
  piece.phenotypes = [];

  const row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

  for (let i = 0; i < seniApp.populationSize; i++) {
    phenotypeElement = createPhenotypeElement(i, '');

    // get the image element
    imageElement = phenotypeElement.getElementsByClassName('phenotype')[0];
    imageElement.addEventListener('load', imageLoadHandler, false);

    row.appendChild(phenotypeElement);

    piece.phenotypes.push({
      phenotypeElement,
      imageElement,
      selected: false,
      imageLoadTimeStamp: 0
    });
  }

  window.addEventListener('popstate', event => {
    console.log('popstate called', event);

    historyUpdateAppState(seniApp, event.state);
    //seniApp.history.back();
    // todo: UI to current position in history object

    const href = document.location.href.split('/');
    console.log(href);
  });
}

function getGallery() {
  const list = document.getElementById('gallery-list');
  list.innerHTML = '';

  const row = document.createElement('div');
  row.className = 'row';
  list.appendChild(row);

  const createGalleryElement = galleryItem => {
    const container = document.createElement('div');

    container.className = 'col s6 m4 l3';
    container.id = `gallery-item-${galleryItem.id}`;

    container.innerHTML = `
      <div class="card">
        <a href="#" class="card-image show-edit">
          <img class="gallery-item-image show-edit"
               src="${galleryItem.image}">
        </a>
        <div class="card-action">
          <span>${galleryItem.name}</span>
        </div>
      </div>
      `;

    return container;
  };

  const url = '/gallery';
  getJSON(url).then(galleryItems => {
    // gets an array of gallery items
    galleryItems.forEach(item => {
      const e = createGalleryElement(item);
      row.appendChild(e);
    });
  }).catch(() => {
    console.error(`cannot connect to ${url}`);
  });
}

function historyUpdateAppState(seniApp, state) {
  // restore the app's current state from state

  console.log('history::updateAppState', state);

  seniApp.currentMode = state.mode;
  showCurrentMode(seniApp);

  switch (seniApp.currentMode) {
  case SeniMode.gallery :
    hideTopNavBar(seniApp);
    break;
  case SeniMode.edit :
    // restore state to the app
    seniApp.piece.script = state.script;
    // update the ui
    showTopNavBar(seniApp);
    timedRenderScript(seniApp, 'renderScript');
    break;
  case SeniMode.evolve :
    // restore state to the app
    seniApp.piece.script = state.script;
    seniApp.piece.genotypes = state.genotypes;
    // todo: restore the selected genotypes
    // update the ui
    showTopNavBar(seniApp);

    showPlaceholderImages(seniApp);
    renderPhenotypes(seniApp);
    break;
  }
}

function historyAdd(seniApp) {

  const state = {
    mode: seniApp.currentMode,
    script: seniApp.piece.script,
    genotypes: seniApp.piece.genotypes
  };

  seniApp.lastState = state;

  const uri = `#${seniModeAsString(seniApp.currentMode)}`;
  history.pushState(state, null, uri);
}

function historyAddSelectedGenotypes(/*selectedGenotypes*/) {

  // keep a copy of the last state added
  // modify that state here
  // call history.replaceState()
 //    console.log('adding selected genotypes to state index', this.stateIndex);
  //    const stateItem = this.stateList[this.stateIndex];
  //    if (stateItem.mode !== SeniMode.evolve) {
  //      return;
  //    }
  //    stateItem.selectedGenotypes = selectedGenotypes;
}

class Piece {
  constructor() {
    this.phenotypes = [];
    // selectedGenotypes is required to remember the previous selection
    // in case of a shuffle
    this.selectedGenotypes = [];
    this.script = undefined;
    this.frontAst = undefined;
    this.backAst = undefined;
    this.traits = undefined;
    this.genotypes = [];
  }
}

function createSeniApp() {
  const canvasElement = document.getElementById('render-canvas');
  const seniApp = {
    currentMode: SeniMode.gallery,
    renderer: new Renderer(canvasElement),
    editor: undefined,
    // the top nav bar across the app
    navbar: undefined,
    // the img destination that shows the rendered script in edit mode
    renderImage: undefined,
    // the resolution of the high res image
    highResolution: [2048, 2048],
    // the 3 main UI areas
    containers: [],
    placeholder: 'img/spinner.gif',
    populationSize: 24,
    mutationRate: 0.1,
    // an immutable var containing the base env for all evaluations
    env: undefined,
    // information about the current piece being created/rendered
    piece: new Piece(),
    // for browser history modification
    lastState: undefined
  };

  seniApp.env = Bind.addBindings(Runtime.createEnv(), seniApp.renderer);
  historyAdd(seniApp);

  return seniApp;
}

const SeniWebApplication = {
  mainFn() {
    resizeContainers();

    const seniApp = createSeniApp();

    polluteGlobalDocument(seniApp);

    setupUI(seniApp);

    getGallery();
    //iterateEnv(seniApp);
  }
};

export default SeniWebApplication;
