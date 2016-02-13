/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
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

import Immutable from 'immutable';

import Bind from './lang/Bind';
import Renderer from './seni/Renderer';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';
import { SeniMode } from './ui/SeniMode';
import History from './ui/History';
import Konsole from './ui/Konsole';
import KonsoleCommander from './ui/KonsoleCommander';
import { addDefaultCommands } from './ui/KonsoleCommands';
import Editor from './ui/Editor';
import { createStore, createInitialState } from './store';
import { startTiming } from './timer';

let gUI = {};
let gRenderer = undefined;
// an immutable var containing the base env for all evaluations
let gEnv = undefined;

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

function getScriptFromEditor() {
  return gUI.editor.getValue();
}

function showButtonsFor(mode) {

  const evalBtn = document.getElementById('eval-btn');
  const evolveBtn = document.getElementById('evolve-btn');
  const renderBtn = document.getElementById('render-btn');
  const nextBtn = document.getElementById('next-btn');
  const shuffleBtn = document.getElementById('shuffle-btn');

  switch (mode) {
  case SeniMode.gallery :
    evalBtn.classList.add('hidden');
    evolveBtn.classList.add('hidden');
    renderBtn.classList.add('hidden');
    nextBtn.classList.add('hidden');
    shuffleBtn.classList.add('hidden');
    break;
  case SeniMode.edit :
    evalBtn.classList.remove('hidden');
    evolveBtn.classList.remove('hidden');
    renderBtn.classList.remove('hidden');
    nextBtn.classList.add('hidden');
    shuffleBtn.classList.add('hidden');
    break;
  case SeniMode.evolve :
    evalBtn.classList.add('hidden');
    evolveBtn.classList.add('hidden');
    renderBtn.classList.add('hidden');
    nextBtn.classList.remove('hidden');
    shuffleBtn.classList.remove('hidden');
    break;
  }
}

function ensureMode(store, mode) {
  return new Promise((resolve, _) => {
    if (store.getState().get('currentMode') === mode) {
      resolve();
      return;
    }

    store.dispatch({type: 'SET_MODE', mode});
    History.pushState(store.getState());

    if (mode === SeniMode.evolve) {
      showCurrentMode(store.getState());
      setupEvolveUI(store).then(() => {
        // make sure that the history for the first evolve generation
        // has the correct genotypes
        History.replaceState(store.getState());
        resolve();
      });
    } else {
      updateUI(store.getState());
      resolve();
    }
  });
}

// function that takes a read-only state and updates the UI
//
function updateUI(state) {

  showCurrentMode(state);

  switch (state.get('currentMode')) {
  case SeniMode.gallery :
    break;
  case SeniMode.edit :
    showScriptInEditor(state);
    timedRenderScript(state);
    break;
  case SeniMode.evolve :
    // will only get here from History.restoreState
    // NOTE: the popstate event listener is handling this case
    break;
  }
}

function renderGenotypeToImage(state, backAst, genotype, imageElement, w, h) {

  const renderer = gRenderer;

  if (w !== undefined && h !== undefined) {
    renderer.preDrawScene(w, h);
  } else {
    renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);
  }

  Runtime.evalAst(gEnv, backAst, genotype);
  renderer.postDrawScene();
  imageElement.src = renderer.getImageData();
}

function renderScript(state, imageElement) {

  const script = state.get('script');
  const frontAst = Runtime.buildFrontAst(script);
  if (frontAst.error) {
    gUI.konsole.log(frontAst.error);
    return;
  }
  const backAst = Runtime.compileBackAst(frontAst.nodes);
  const traits = Genetic.buildTraits(backAst);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  renderGenotypeToImage(state, backAst, genotype, imageElement);
}

function timedRenderScript(state) {
  const stopFn = startTiming(`renderScript-${state.get('scriptHash')}`);
  renderScript(state, gUI.renderImage);
  stopFn();
}

function addClickEvent(id, fn) {

  const element = document.getElementById(id);

  if (element) {
    element.addEventListener('click', fn);
  } else {
    console.log('cannot addClickEvent for', id);
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

function renderHighRes(state, genotype) {
  const container = document.getElementById('high-res-container');
  const loader = document.getElementById('high-res-loader');
  const image = document.getElementById('high-res-image');

  container.classList.remove('hidden');
  loader.classList.remove('hidden');
  image.classList.add('hidden');

  setTimeout(() => {
    const script = state.get('script');
    const frontAst  = Runtime.buildFrontAst(script);
    if (frontAst.error) {
      gUI.konsole.log(frontAst.error);
      image.classList.remove('hidden');
      loader.classList.add('hidden');
      return;
    }
    const backAst = Runtime.compileBackAst(frontAst.nodes);

    if (genotype === undefined) {
      const traits = Genetic.buildTraits(backAst);
      genotype = Genetic.createGenotypeFromInitialValues(traits);
    }

    const [width, height] = state.get('highResolution');
    renderGenotypeToImage(state, backAst, genotype, image,
                          width, height);

    image.classList.remove('hidden');

    const link = document.getElementById('high-res-link');
    link.href = image.src;

    loader.classList.add('hidden');
  }, 100);
}

function showEditFromEvolve(store, element) {
  return new Promise((resolve, reject) => {
    const [index, _] = getPhenoIdFromDom(element);
    if (index !== -1) {
      const genotypes = store.getState().get('genotypes');
      const genotype = genotypes.get(index);

      const state = store.getState();
      const frontAst  = Runtime.buildFrontAst(state.get('script'));
      if (frontAst.error) {
        gUI.konsole.log(frontAst.error);
        reject();
        return;
      } else {
        const script = Runtime.unparse(frontAst.nodes, genotype);

        store.dispatch({type: 'SET_SCRIPT', script});

        ensureMode(store, SeniMode.edit).then(() => {
          resolve();
        });
      }
    } else {
      resolve();
    }
  });
}

function renderGeneration(state) {
  return new Promise((resolve, reject) => {

    const script = state.get('script');
    const scriptHash = state.get('scriptHash');

    const stopTiming = startTiming(`renderGeneration-${scriptHash}`);

    const frontAst = Runtime.buildFrontAst(script);
    if (frontAst.error) {
      gUI.konsole.log(frontAst.error);
      reject();
      return;
    }

    const backAst = Runtime.compileBackAst(frontAst.nodes);
    let i = 0;

    setTimeout(function go() {
      // stop generating new phenotypes if we've reached the desired
      // population or the user has switched to edit mode
      const phenotypes = gUI.phenotypes;
      const genotypes = state.get('genotypes');
      if (i < phenotypes.size && state.get('currentMode') === SeniMode.evolve) {

        const genotype = genotypes.get(i);
        const imageElement = phenotypes.getIn([i, 'imageElement']);

        renderGenotypeToImage(state,
                              backAst,
                              genotype,
                              imageElement);
        i++;
        setTimeout(go);
      } else {
        stopTiming();
        resolve();
      }
    });
  });
}

function showPlaceholderImages(state) {

  const placeholder = state.get('placeholder');
  const populationSize = state.get('populationSize');
  const phenotypes = gUI.phenotypes;

  for (let i = 0; i < populationSize; i++) {
    const imageElement = phenotypes.getIn([i, 'imageElement']);
    imageElement.src = placeholder;
  }
}

// update the selected phenotypes in the evolve screen according to the
// values in selectedIndices
function updateSelectionUI(state) {

  const selectedIndices = state.get('selectedIndices');
  const populationSize = state.get('populationSize');
  const phenotypes = gUI.phenotypes;

  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes.getIn([i, 'phenotypeElement']);
    element.classList.remove('selected');
  }

  selectedIndices.forEach(i => {
    const element = gUI.phenotypes.getIn([i, 'phenotypeElement']);
    element.classList.add('selected');
    return true;
  });
}

function onNextGen(store) {

  // get the selected genotypes for the next generation
  const populationSize = store.getState().get('populationSize');
  const phenotypes = gUI.phenotypes;
  let selectedIndices = new Immutable.List();

  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes.getIn([i, 'phenotypeElement']);
    if (element.classList.contains('selected')) {
      selectedIndices = selectedIndices.push(i);
    }
  }

  store.dispatch({type: 'SET_SELECTED_INDICES', selectedIndices});

  if (selectedIndices.size === 0) {
    // no phenotypes were selected
    return;
  }

  // update the last history state
  History.replaceState(store.getState());

  showPlaceholderImages(store.getState());

  store.dispatch({type: 'NEXT_GENERATION', rng: 42});

  History.pushState(store.getState());

  // render the genotypes
  updateSelectionUI(store.getState());
  renderGeneration(store.getState());
}

function createPhenotypeElement(id, placeholderImage) {

  const container = document.createElement('div');

  container.className = 'card-holder';
  container.id = `pheno-${id}`;
  container.innerHTML = `
      <a href="#">
        <img class="card-image phenotype"
             data-id="${id}" src="${placeholderImage}">
      </a>
      <div class="card-action">
        <a href="#" class="render left-side">Render</a>
        <a href="#" class="edit right-side">Edit</a>
      </div>`;

  return container;
}

// invoked when the evolve screen is displayed after the edit screen
function setupEvolveUI(store) {
  return new Promise((resolve, _) => {
    afterLoadingPlaceholderImages(store.getState()).then(() => {
      store.dispatch({type: 'INITIAL_GENERATION'});
      // render the phenotypes
      updateSelectionUI(store.getState());
      return renderGeneration(store.getState());
    }).then(resolve);
  });
}

// invoked when restoring the evolve screen from the history api
function restoreEvolveUI(store) {
  return new Promise((resolve, _) => { // todo: implement reject
    afterLoadingPlaceholderImages(store.getState()).then(() => {
      // render the phenotypes
      updateSelectionUI(store.getState());
      return renderGeneration(store.getState());
    }).then(resolve);
  });
}

// needs the store since imageLoadHandler rebinds store.getState()
// on every image load
//
function afterLoadingPlaceholderImages(state) {
  const allImagesLoadedSince = timeStamp => {
    const phenotypes = gUI.phenotypes;

    return phenotypes.every(phenotype => {
      const imageElement = phenotype.get('imageElement');
      const loaded = imageElement.getAttribute('data-image-load-timestamp');
      return loaded > timeStamp;
    });
  };

  const initialTimeStamp = Date.now();

  showPlaceholderImages(state);

  return new Promise((resolve, _) => { // todo: implement reject
    setTimeout(function go() {
      // wait until all of the placeholder load events have been received
      // otherwise there may be image sizing issues, especially with the
      // first img element
      if (allImagesLoadedSince(initialTimeStamp)) {
        resolve(state);
      } else {
        setTimeout(go, 20);
      }
    });
  });
}

function showCurrentMode(state) {

  // show the current container, hide the others
  const containers = gUI.containers;
  const currentMode = state.get('currentMode');

  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    containers.get(i).className = i === currentMode ? '' : 'hidden';
  }
  showButtonsFor(currentMode);
}

function showScriptInEditor(state) {

  const editor = gUI.editor;

  editor.getDoc().setValue(state.get('script'));
  editor.refresh();
}

function showEditFromGallery(store, element) {
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

  return new Promise((resolve, reject) => {
    const [index, _] = getGalleryItemIdFromDom(element);
    if (index !== -1) {
      const url = `/gallery/${index}`;

      get(url).catch(() => {
        reject(Error(`cannot connect to ${url}`));
      }).then(data => {
        store.dispatch({type: 'SET_SCRIPT', script: data});
        return ensureMode(store, SeniMode.edit);
      }).then(resolve);
    } else {
      resolve();
    }
  });
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


function createKonsole(env, element) {

  const konsole = new Konsole(element, {
    prompt: '> ',
    historyLabel: 'cs-console-demo',
    syntax: 'javascript',
    initialValue: 'This is starting content\nalong with multi-lines!\n',
    welcomeMessage: 'Welcome to the cs console demo',
    autoFocus: true,
    theme: 'konsole'
  });

  const commander = new KonsoleCommander();
  addDefaultCommands(env, commander);

  konsole.initCallbacks({
    commandValidate(line) {
      return line.length > 0;
    },
    commandHandle(line, report, prompt) {
      commander.commandHandle(line, report, prompt);
    }
  });

  return konsole;
}

function createEditor(store, editorTextArea) {

  const blockIndent = function(editor, from, to) {
    editor.operation(() => {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  const extraKeys = {
    'Ctrl-E': () => {
      store.dispatch({type: 'SET_SCRIPT', script: getScriptFromEditor()});
      timedRenderScript(store.getState());
      return false;
    },
    // make ctrl-m a noop, otherwise invoking the konsole will result in
    // deleting a line from the editor
    'Ctrl-M': () => false,
    'Ctrl-I': () => {
      const editor = gUI.editor;
      const konsole = gUI.konsole;
      const numLines = editor.doc.size;
      blockIndent(editor, 0, numLines);
      konsole.log(`indenting ${numLines} lines`);
      return false;
    }
  };

  return Editor.createEditor(editorTextArea, {
    theme: 'default',
    extraKeys
  });
}

function setupUI(store) {

  const d = document;
  const konsoleElement = d.getElementById('konsole');
  const editorTextArea = d.getElementById('edit-textarea');

  gUI = {
    // the 3 main UI areas, stored in an Immutable.List
    containers: new Immutable.List([d.getElementById('gallery-container'),
                                    d.getElementById('edit-container'),
                                    d.getElementById('evolve-container')]),
    // the top nav bar across the state
    navbar: d.getElementById('seni-navbar'),
    // the img destination that shows the rendered script in edit mode
    renderImage: d.getElementById('render-img'),
    // console CodeMirror element in the edit screen
    konsole: createKonsole(gEnv, konsoleElement),
    editor: createEditor(store, editorTextArea)
  };

  konsoleElement.style.height = `0%`;

  showButtonsFor(SeniMode.gallery);

  addClickEvent('home', event => {
    ensureMode(store, SeniMode.gallery);
    event.preventDefault();
  });

  addClickEvent('evolve-btn', event => {
    // get the latest script from the editor
    store.dispatch({type: 'SET_SCRIPT', script: getScriptFromEditor()});
    History.replaceState(store.getState());
    ensureMode(store, SeniMode.evolve);
    event.preventDefault();
  });

  addClickEvent('render-btn', event => {
    renderHighRes(store.getState());
    event.preventDefault();
  });

  addClickEvent('shuffle-btn', event => {
    showPlaceholderImages(store.getState());
    store.dispatch({type: 'SHUFFLE_GENERATION', rng: 11});
    updateSelectionUI(store.getState());
    renderGeneration(store.getState());
    event.preventDefault();
  });

  addClickEvent('eval-btn', () => {
    store.dispatch({type: 'SET_SCRIPT', script: getScriptFromEditor()});
    timedRenderScript(store.getState());
  });

  addClickEvent('gallery-container', event => {
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      showEditFromGallery(store, target).catch(error => {
        console.error(error);
      });
    }
    event.preventDefault();
  });

  addClickEvent('evolve-container', event => {
    const target = event.target;
    const [index, phenoElement] = getPhenoIdFromDom(target);

    if (target.classList.contains('render')) {
      if (index !== -1) {
        const genotypes = store.getState().get('genotypes');
        const genotype = genotypes.get(index);
        renderHighRes(store.getState(), genotype);
      }
    } else if (target.classList.contains('edit')) {
      showEditFromEvolve(store, target);
    } else {
      if (index !== -1) {
        phenoElement.classList.toggle('selected');
      }
    }
    event.preventDefault();
  });

  addClickEvent('next-btn', () => {
    onNextGen(store);
  });

  addClickEvent('high-res-download', event => {
    const highResLink = document.getElementById('high-res-link');

    // remove target='_blank' and add a download attribute
    highResLink.removeAttribute('target');
    highResLink.setAttribute('download', 'seni-image.png');

    highResLink.click();

    // restore attributes
    highResLink.removeAttribute('download');
    highResLink.setAttribute('target', '_blank');

    event.preventDefault();
  });

  addClickEvent('high-res-close', event => {
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.add('hidden');
    event.preventDefault();
  });

  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    if (event.ctrlKey && event.keyCode === dKey &&
        store.getState().get('currentMode') === SeniMode.evolve) {
      event.preventDefault();
      onNextGen(store);
    }
  }, false);

  // invoked on every load event for an img tag
  const imageLoadHandler = event => {
    event.target.setAttribute('data-image-load-timestamp', event.timeStamp);
  };

  // setup the evolve-container
  const evolveGallery = document.getElementById('evolve-gallery');
  evolveGallery.innerHTML = '';

  const row = document.createElement('div');
  row.className = 'cards';
  evolveGallery.appendChild(row);

  let phenotypeElement, imageElement;

  const populationSize = store.getState().get('populationSize');
  const phenotypes = [];
  for (let i = 0; i < populationSize; i++) {
    phenotypeElement = createPhenotypeElement(i, '');

    // get the image element
    imageElement = phenotypeElement.getElementsByClassName('phenotype')[0];
    imageElement.addEventListener('load', imageLoadHandler, false);
    imageElement.setAttribute('data-image-load-timestamp', 0);

    row.appendChild(phenotypeElement);

    phenotypes.push(new Immutable.Map({
      phenotypeElement,
      imageElement
    }));
  }

  gUI.phenotypes = new Immutable.List(phenotypes);

  window.addEventListener('popstate', event => {
    if (event.state) {
      const savedState = History.restoreState(event.state);
      store.dispatch({type: 'SET_STATE', state: savedState});
      updateUI(store.getState());
      if (store.getState().get('currentMode') === SeniMode.evolve) {
        restoreEvolveUI(store);
      }
    } else {
      // no event.state so behave as if the user has visited
      // the '/' of the state
      ensureMode(store, SeniMode.gallery);
    }
  });

  let konsoleToggle = 0;

  function toggleKonsole() {
    const konsolePanel = document.getElementById('konsole');
    const konsoleButton = document.getElementById('console-btn');

    konsoleToggle = 1 - konsoleToggle;
    if (konsoleToggle === 1) {
      konsolePanel.style.height = '50%';
      konsoleButton.textContent = 'Hide Console';
      gUI.konsole.focus();
    } else {
      gUI.editor.focus();
      konsolePanel.style.height = '0%';
      konsoleButton.textContent = 'Show Console';
    }
    gUI.konsole.refresh();
    gUI.editor.refresh();
  }

  document.onkeydown = evt => {
    evt = evt || window.event;

    // Ctrl-M
    if (evt.ctrlKey && evt.keyCode == 77) {
      toggleKonsole();
    }
  };

  addClickEvent('console-btn', toggleKonsole);

  return store;
}

function getGallery() {

  const createGalleryElement = galleryItem => {
    const container = document.createElement('div');

    container.className = 'card-holder';
    container.id = `gallery-item-${galleryItem.id}`;

    container.innerHTML = `
      <a href="#" class="show-edit">
        <img class="card-image show-edit"
             src="${galleryItem.image}">
      </a>
      <div class="card-action">
        <span>${galleryItem.name}</span>
      </div>`;

    return container;
  };

  return new Promise((resolve, reject) => {
    const list = document.getElementById('gallery-container');
    list.innerHTML = '';

    const row = document.createElement('div');
    row.className = 'cards';
    list.appendChild(row);

    const url = '/gallery';
    getJSON(url).then(galleryItems => {
      // gets an array of gallery items
      galleryItems.forEach(item => {
        const e = createGalleryElement(item);
        row.appendChild(e);
      });
      resolve();
    }).catch(() => {
      reject(Error(`cannot connect to ${url}`));
    });
  });
}

// stops the konsole from briefly flashing at state startup
// probably better to remove this and replace with some other
// sort of CSS cleverness. (resorting to this since a CSS rule
// of 'position: fixed;height:0;' for #konsole screws up Chrome
// and requires a restart)
function removeKonsoleInvisibility() {
  const k = document.getElementById('konsole');
  k.classList.remove('invisible');
}

export default function main() {
  resizeContainers();

  gRenderer = new Renderer(document.getElementById('render-canvas'));
  gEnv = Bind.addBindings(
    Bind.addClassicBindings(
      Bind.addSpecialDebugBindings(
        Bind.addSpecialBindings(
          Runtime.createEnv()))),
    gRenderer);

  const state = createInitialState();
  const store = createStore(state);
  setupUI(store);

  getGallery()
    .then(removeKonsoleInvisibility)
    .catch(error => console.error(error));
}
