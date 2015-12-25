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

import Immutable from 'immutable';

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
  const editor = seniApp.get('editor');
  return seniApp.set('pieceScript', editor.getValue());
}

function ensureMode(seniAppContainer, mode) {
  const seniApp = seniAppContainer.seniApp;
  if (seniApp.get('currentMode') === mode) {
    return seniAppContainer;
  }

  seniAppContainer.seniApp = seniApp.set('currentMode', mode);
  // todo: historyAdd(seniApp) ????

  // todo: ideally this shouldn't change seniApp
  seniAppContainer = updateUI(seniAppContainer);

  return seniAppContainer;
}

// function that takes a read-only seniApp and updates the UI
//
function updateUI(seniAppContainer) {
  const seniApp = seniAppContainer.seniApp;
  showCurrentMode(seniApp);

  switch (seniApp.get('currentMode')) {
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
    seniAppContainer.seniApp = getScriptFromEditor(seniApp);
    // if it's a change of mode into the SeniMode.evolve
    seniAppContainer = setupEvolveUI(seniAppContainer);
    // else if there's been a change in selection ???
    break;
  }

  return seniAppContainer;
}

// search the children of seniApp.navbar for elements with class 'klass'
// then add 'addClass' to them
function addNavbarClass(seniApp, klass, addClass) {
  const navbar = seniApp.get('navbar');
  const es = navbar.getElementsByClassName(klass);
  for (let i = 0; i < es.length; i++) {
    es[i].classList.add(addClass);
  }
}

function removeNavbarClass(seniApp, klass, removeClass) {
  const navbar = seniApp.get('navbar');
  const es = navbar.getElementsByClassName(klass);
  for (let i = 0; i < es.length; i++) {
    es[i].classList.remove(removeClass);
  }
}

function renderGenotypeToImage(seniApp, ast, genotype, imageElement, w, h) {

  const renderer = seniApp.get('renderer');

  if (w !== undefined && h !== undefined) {
    renderer.preDrawScene(w, h);
  } else {
    renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);
  }

  Runtime.evalAst(seniApp.get('env'), ast, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
}

function renderScript(seniApp) {
  const imageElement = seniApp.get('renderImage');

  const script = seniApp.get('pieceScript');
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
    const pieceGenotypes = seniApp.get('pieceGenotypes');
    const genotype = pieceGenotypes[index];
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.remove('invisible');
    const pieceScript = seniApp.get('pieceScript');
    const frontAst = Runtime.buildFrontAst(pieceScript);
    const backAst = Runtime.compileBackAst(frontAst);

    const imageElement = document.getElementById('high-res-image');
    const [width, height] = seniApp.get('highResolution');
    renderGenotypeToImage(seniApp, backAst, genotype, imageElement,
                          width, height);

    const holder = document.getElementById('holder');
    imageElement.style.height = `${holder.clientHeight}px`;
    imageElement.style.width = `${holder.clientWidth}px`;

    const linkElement = document.getElementById('high-res-link');
    linkElement.href = imageElement.src;
  }
}

function showEditFromEvolve(seniAppContainer, element) {

  const seniApp = seniAppContainer.seniApp;
  /* eslint-disable no-unused-vars */
  const [index, _] = getPhenoIdFromDom(element);
  /* eslint-enable no-unused-vars */

  if (index !== -1) {
    const pieceGenotypes = seniApp.get('pieceGenotypes');
    const genotype = pieceGenotypes[index];
    const frontAst = seniApp.get('pieceFrontAst');

    const script = Runtime.unparse(frontAst, genotype);

    seniAppContainer.seniApp = seniApp.set('pieceScript', script);

    seniAppContainer = ensureMode(seniAppContainer, SeniMode.edit);
  }

  return seniAppContainer;
}

function toggleSelection(seniApp, element) {
  const [index, e] = getPhenoIdFromDom(element);
  if (index !== -1) {
    const cardImage = e.getElementsByClassName('card-image')[0];
    cardImage.classList.toggle('selected');

    const piecePhenotypes = seniApp.get('piecePhenotypes');
    const c = piecePhenotypes[index];
    c.selected = !c.selected;
  }
}

function renderPhenotypes(seniApp) {

  let i = 0;
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to edit mode
    const piecePhenotypes = seniApp.get('piecePhenotypes');
    const pieceGenotypes = seniApp.get('pieceGenotypes');
    if (i < piecePhenotypes.length &&
        seniApp.get('currentMode') === SeniMode.evolve) {

      const genotype = pieceGenotypes[i];
      const imageElement = piecePhenotypes[i].imageElement;

      renderGenotypeToImage(seniApp,
                            seniApp.get('pieceBackAst'),
                            genotype,
                            imageElement);
      i++;
      setTimeout(go);
    }
  });
}

function showPlaceholderImages(seniApp) {
  const placeholder = seniApp.get('placeholder');
  const populationSize = seniApp.get('populationSize');
  const piecePhenotypes = seniApp.get('piecePhenotypes');
  for (let i = 0; i < populationSize; i++) {
    const imageElement = piecePhenotypes[i].imageElement;
    imageElement.src = placeholder;
  }
}

function createInitialGenotypePopulation(seniApp, populationSize) {
  // add genotypes to the containers
  let genotype;
  const random = (new Date()).toGMTString();
  const pieceTraits = seniApp.get('pieceTraits');
  const pieceGenotypes = [];

  for (let i = 0; i < populationSize; i++) {
    if (i === 0) {
      genotype = Genetic.createGenotypeFromInitialValues(pieceTraits);
    } else {
      genotype = Genetic.createGenotypeFromTraits(pieceTraits, i + random);
    }
    // todo: is this the right way of updating pieceGenotypes
    pieceGenotypes.push(genotype);
  }

  seniApp = seniApp.set('pieceGenotypes', pieceGenotypes);
  return seniApp;
}

function genotypesFromSelectedPhenotypes(seniApp) {

  showPlaceholderImages(seniApp);

  const pieceSelectedGenotypes = seniApp.get('pieceSelectedGenotypes');

  if (pieceSelectedGenotypes.length === 0) {
    // if this is the first generation and nothing has been selected
    // just randomize all of the phenotypes
    seniApp = createInitialGenotypePopulation(seniApp, seniApp.populationSize);
  } else {
    seniApp = seniApp.set('pieceGenotypes', Genetic.nextGeneration(
      seniApp.get('pieceSelectedGenotypes'),
      seniApp.get('populationSize'),
      seniApp.get('mutationRate'),
      seniApp.get('pieceTraits')));
  }
  historyAdd(seniApp);

  // render the genotypes
  renderPhenotypes(seniApp);

  // clean up the dom and clear the selected state
  const populationSize = seniApp.get('populationSize');
  const piecePhenotypes = seniApp.get('piecePhenotypes');
  for (let i = 0; i < populationSize; i++) {
    if (piecePhenotypes[i].selected === true) {
      const element = piecePhenotypes[i].phenotypeElement;
      const cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.remove('selected');
    }
    piecePhenotypes[i].selected = false;
  }

  return seniApp;
}

function onNextGen(seniApp) {
  // get the selected genotypes for the next generation

  const populationSize = seniApp.get('populationSize');
  const pieceSelectedGenotypes = [];
  const piecePhenotypes = seniApp.get('piecePhenotypes');
  const pieceGenotypes = seniApp.get('pieceGenotypes');

  for (let i = 0; i < populationSize; i++) {
    if (piecePhenotypes[i].selected === true) {
      pieceSelectedGenotypes.push(pieceGenotypes[i]);
    }
  }

  seniApp = seniApp.set('pieceSelectedGenotypes', pieceSelectedGenotypes);

  if (pieceSelectedGenotypes.length === 0) {
    // no phenotypes were selected
    return seniApp;
  }

  // todo: implement
  historyAddSelectedGenotypes(seniApp.get('pieceSelectedGenotypes'));
  seniApp = genotypesFromSelectedPhenotypes(seniApp);

  return seniApp;
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

function setupEvolveUI(seniAppContainer) {

  let seniApp = seniAppContainer.seniApp;

  const allImagesLoadedSince = function(timeStamp) {
    const populationSize = seniApp.get('populationSize');
    const piecePhenotypes = seniApp.get('piecePhenotypes');
    for (let i = 0; i < populationSize; i++) {
      if (piecePhenotypes[i].imageLoadTimeStamp < timeStamp) {
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

      const pieceScript = seniApp.get('pieceScript');
      seniApp = seniApp.set('pieceFrontAst',
                            Runtime.buildFrontAst(pieceScript));

      const pieceFrontAst = seniApp.get('pieceFrontAst');
      seniApp = seniApp.set('pieceBackAst',
                            Runtime.compileBackAst(pieceFrontAst));

      const pieceBackAst = seniApp.get('pieceBackAst');
      seniApp = seniApp.set('pieceTraits',
                            Genetic.buildTraits(pieceBackAst));

      const populationSize = seniApp.get('populationSize');
      seniApp = createInitialGenotypePopulation(seniApp, populationSize);

      // render the phenotypes
      renderPhenotypes(seniApp);

      // the reason for passing seniAppContainer into setupEvolveUI is so
      // that it references the correct seniApp after the conditions in this
      // timeout have been met
      seniAppContainer.seniApp = seniApp;

    } else {
      setTimeout(go, 20);
    }
  });
  return seniAppContainer;
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
  const containers = seniApp.get('containers');
  const currentMode = seniApp.get('currentMode');
  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    containers[i].className = i === currentMode ? '' : 'hidden';
  }
}

function showScriptInEditor(seniApp) {
  const editor = seniApp.get('editor');
  editor.getDoc().setValue(seniApp.get('pieceScript'));
  editor.refresh();
}

function showEditFromGallery(seniAppContainer, element) {

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
      let sa = seniAppContainer.seniApp;
      sa = sa.set('pieceScript', data);
      seniAppContainer.seniApp = sa;
      seniAppContainer = ensureMode(seniAppContainer, SeniMode.edit);
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

function polluteGlobalDocument(seniAppContainer) {
  const seniApp = seniAppContainer.seniApp;

  document.seni = {};
  document.seni.title = Trivia.getTitle;
  document.seni.help = function(name, showDefaultArgs = false) {
    const v = seniApp.getIn(['env', name]);
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
    const env = seniApp.get('env');
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

function setupUI(seniAppContainer) {
  let sa = seniAppContainer.seniApp;
  const d = document;

  sa = sa.set('navbar', document.getElementById('seni-navbar'));
  sa = sa.set('renderImage', document.getElementById('render-img'));
  sa = sa.set('containers', [document.getElementById('gallery-container'),
                             document.getElementById('edit-container'),
                             document.getElementById('evolve-container')]);

  // hide the navbar links because we start off in gallery mode
  addNavbarClass(sa, 'to-gallery', 'hidden');
  addNavbarClass(sa, 'to-edit', 'hidden');
  addNavbarClass(sa, 'to-evolve', 'hidden');

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
      seniAppContainer.seniApp = getScriptFromEditor(seniAppContainer.seniApp);
      timedRenderScript(seniAppContainer.seniApp, 'renderScript');
      return false;
    },
    'Ctrl-D': () => false,
    'Ctrl-I': () => {
      const editor = seniAppContainer.seniApp.get('editor');
      const numLines = editor.doc.size;
      blockIndent(editor, 0, numLines);
      console.log('indenting', numLines, 'lines');
      return false;
    }
  };

  const textArea = d.getElementById('codemirror-textarea');
  sa = sa.set('editor', codeMirror.fromTextArea(textArea, config));

  const galleryModeHandler = event => {
    seniAppContainer = ensureMode(seniAppContainer, SeniMode.gallery);
    event.preventDefault();
  };

  const evolveModeHandler = event => {
    seniAppContainer = ensureMode(seniAppContainer, SeniMode.evolve);
    event.preventDefault();
  };

  const editModeHandler = event => {
    seniAppContainer = ensureMode(seniAppContainer, SeniMode.edit);
    event.preventDefault();
  };


  addClickEvent('evolve-mode-icon', evolveModeHandler);
  addClickEventForClass('to-evolve', evolveModeHandler);

  addClickEvent('edit-mode-icon', editModeHandler);
  addClickEventForClass('to-edit', editModeHandler);

  addClickEventForClass('to-gallery', galleryModeHandler);

  addClickEvent('shuffle-icon', event => {
    const seniApp = seniAppContainer.seniApp;
    seniAppContainer.seniApp = genotypesFromSelectedPhenotypes(seniApp);
    event.preventDefault();
  });

  addClickEvent('action-eval', () => {
    let seniApp = seniAppContainer.seniApp;
    const editor = seniApp.get('editor');
    seniApp = seniApp.set('pieceScript', editor.getValue());
    timedRenderScript(seniApp, 'renderScript');
    seniAppContainer.seniApp = seniApp;
  });

  addClickEvent('action-add', () => {
    const seniApp = seniAppContainer.seniApp;
    seniAppContainer.seniApp = seniApp.set('pieceScript', '');
    seniAppContainer = ensureMode(seniAppContainer, SeniMode.edit);
  });

  addClickEvent('gallery-list', event => {
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      showEditFromGallery(seniAppContainer, target);
    }
    event.preventDefault();
  });

  addClickEvent('phenotype-gallery', event => {
    const seniApp = seniAppContainer.seniApp;

    const target = event.target;
    if (target.classList.contains('render')) {
      renderHighRes(seniApp, target);
    } else if (target.classList.contains('edit')) {
      seniAppContainer = showEditFromEvolve(seniAppContainer, target);
    } else {
      toggleSelection(seniApp, target);
    }
    event.preventDefault();
  });

  addClickEvent('action-next-gen', () => {
    seniAppContainer.seniApp = onNextGen(seniAppContainer.seniApp);
  });

  addClickEvent('high-res-close', event => {
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.add('invisible');

    event.preventDefault();
  });

  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    const seniApp = seniAppContainer.seniApp;
    if (event.ctrlKey && event.keyCode === dKey &&
        seniApp.get('currentMode') === SeniMode.evolve) {
      event.preventDefault();
      seniAppContainer.seniApp = onNextGen(seniApp);
    }
  }, false);

  // invoked on every load event for an img tag
  const imageLoadHandler = event => {
    const sac = seniAppContainer;
    const seniApp = sac.seniApp;
    const imageId = event.target.getAttribute('data-id');
    const piecePhenotypes = seniApp.get('piecePhenotypes');
    piecePhenotypes[imageId].imageLoadTimeStamp = event.timeStamp;
  };



  const gallery = document.getElementById('phenotype-gallery');
  gallery.innerHTML = '';

  let phenotypeElement, imageElement;
  sa = sa.set('piecePhenotypes', []);

  const row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

  const populationSize = sa.get('populationSize');
  const piecePhenotypes = [];
  for (let i = 0; i < populationSize; i++) {
    phenotypeElement = createPhenotypeElement(i, '');

    // get the image element
    imageElement = phenotypeElement.getElementsByClassName('phenotype')[0];
    imageElement.addEventListener('load', imageLoadHandler, false);

    row.appendChild(phenotypeElement);

    piecePhenotypes.push({
      phenotypeElement,
      imageElement,
      selected: false,
      imageLoadTimeStamp: 0
    });
  }

  sa = sa.set('piecePhenotypes', piecePhenotypes);

  window.addEventListener('popstate', event => {
    console.log('popstate called', event);

    historyUpdateAppState(sa, event.state);
    //sa.history.back();
    // todo: UI to current position in history object

    const href = document.location.href.split('/');
    console.log(href);
  });

  seniAppContainer.seniApp = sa;

  return seniAppContainer;
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


// TODO: make this work, also don't forget that seniApp is being modified
function historyUpdateAppState(seniApp, state) {
  // restore the app's current state from state

  console.log('history::updateAppState', state);

  seniApp.currentMode = state.mode;
  showCurrentMode(seniApp);

  switch (seniApp.get('currentMode')) {
  case SeniMode.gallery :
    hideTopNavBar(seniApp);
    break;
  case SeniMode.edit :
    // restore state to the app
    seniApp = seniApp.set('pieceScript', state.script);
    // update the ui
    showTopNavBar(seniApp);
    timedRenderScript(seniApp, 'renderScript');
    break;
  case SeniMode.evolve :
    // restore state to the app
    seniApp.pieceScript = state.script;
    seniApp.pieceGenotypes = state.genotypes;
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
    script: seniApp.pieceScript,
    genotypes: seniApp.pieceGenotypes
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

/**
 * Creates the immutable SeniApp
 *
 * @private
 * @returns {Immutable Map} a basic SeniApp with a valid renderer and env
 */
function createSeniApp() {
  let seniApp = Immutable.fromJS({
    currentMode: SeniMode.gallery,
    renderer: undefined,
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
    piecePhenotypes: [],
    // selectedGenotypes is required to remember the previous selection
    // in case of a shuffle
    pieceSelectedGenotypes: [],
    pieceScript: undefined,
    pieceFrontAst: undefined,
    pieceBackAst: undefined,
    pieceTraits: undefined,
    pieceGenotypes: [],

    // for browser history modification
    lastState: undefined
  });

  const canvasElement = document.getElementById('render-canvas');
  const renderer = new Renderer(canvasElement);
  seniApp = seniApp.set('renderer', renderer);

  const bindings = Bind.addBindings(Runtime.createEnv(), renderer);
  seniApp = seniApp.set('env', bindings);

  // historyAdd(seniApp); // todo: re-instate this once immutable is working

  return {seniApp};
}

const SeniWebApplication = {
  mainFn() {
    resizeContainers();

    let seniAppContainer = createSeniApp();

    polluteGlobalDocument(seniAppContainer);
    seniAppContainer = setupUI(seniAppContainer);
    getGallery();
  }
};

export default SeniWebApplication;
