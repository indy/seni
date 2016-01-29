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

import Renderer from './seni/Renderer';
import Util from './seni/Util';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';
import { SeniMode } from './ui/SeniMode';
import History from './ui/History';
import Konsole from './ui/Konsole';
import KonsoleCommander from './ui/KonsoleCommander';
import { addDefaultCommands } from './ui/KonsoleCommands';
import Editor from './ui/Editor';
import { createStore, createInitialState } from './StateContainer';

let gUI = {};
let gRenderer = undefined;

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
  const nextBtn = document.getElementById('next-btn');
  const shuffleBtn = document.getElementById('shuffle-btn');

  switch (mode) {
  case SeniMode.gallery :
    evalBtn.classList.add('hidden');
    evolveBtn.classList.add('hidden');
    nextBtn.classList.add('hidden');
    shuffleBtn.classList.add('hidden');
    break;
  case SeniMode.edit :
    evalBtn.classList.remove('hidden');
    evolveBtn.classList.remove('hidden');
    nextBtn.classList.add('hidden');
    shuffleBtn.classList.add('hidden');
    break;
  case SeniMode.evolve :
    evalBtn.classList.add('hidden');
    evolveBtn.classList.add('hidden');
    nextBtn.classList.remove('hidden');
    shuffleBtn.classList.remove('hidden');
    break;
  }
}

function ensureMode(store, mode) {
  return new Promise((resolve, _) => {
    if (store.getState().getIn(['saveState', 'currentMode']) === mode) {
      resolve();
      return;
    }

    store.dispatch({type: 'SET_MODE', mode});
    History.pushState(store.getState().get('saveState'));

    if (mode === SeniMode.evolve) {
      showCurrentMode(store.getState());
      setupEvolveUI(store).then(() => {
        // make sure that the history for the first evolve generation
        // has the correct genotypes
        History.replaceState(store.getState().get('saveState'));
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

  switch (state.getIn(['saveState', 'currentMode'])) {
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

  Runtime.evalAst(state.get('env'), backAst, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
}

function renderScript(state) {
  const imageElement = gUI.renderImage;

  const script = state.getIn(['saveState', 'script']);
  const frontAst = Runtime.buildFrontAst(script);
  const backAst = Runtime.compileBackAst(frontAst);
  const traits = Genetic.buildTraits(backAst);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  renderGenotypeToImage(state, backAst, genotype, imageElement);
}

function timedRenderScript(state) {
  Util.withTiming('rendered', () => renderScript(state), gUI.konsole);
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

function renderHighRes(state, element) {
  console.log('renderHighRes');

  const [index, _] = getPhenoIdFromDom(element);

  if (index !== -1) {
    const genotypes = state.getIn(['saveState', 'genotypes']);
    const genotype = genotypes.get(index);
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.remove('hidden');
    const script = state.getIn(['saveState', 'script']);
    const frontAst = Runtime.buildFrontAst(script);
    const backAst = Runtime.compileBackAst(frontAst);

    const imageElement = document.getElementById('high-res-image');
    const [width, height] = state.get('highResolution');
    renderGenotypeToImage(state, backAst, genotype, imageElement,
                          width, height);

    const linkElement = document.getElementById('high-res-link');
    linkElement.href = imageElement.src;
  }
}

function showEditFromEvolve(store, element) {
  return new Promise((resolve, _reject) => {
    const [index, _] = getPhenoIdFromDom(element);
    if (index !== -1) {
      const genotypes = store.getState().getIn(['saveState', 'genotypes']);
      const genotype = genotypes.get(index);
      const frontAst = store.getState().get('frontAst');

      const script = Runtime.unparse(frontAst, genotype);

      store.dispatch({type: 'SET_SCRIPT', script});

      ensureMode(store, SeniMode.edit).then(() => {
        resolve();
      });
    } else {
      resolve();
    }
  });
}

function renderPhenotypes(state) {

  let i = 0;
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to edit mode
    const phenotypes = state.get('phenotypes');
    const genotypes = state.getIn(['saveState', 'genotypes']);
    if (i < phenotypes.size &&
        state.getIn(['saveState', 'currentMode']) === SeniMode.evolve) {

      const genotype = genotypes.get(i);
      const imageElement = phenotypes.getIn([i, 'imageElement']);

      renderGenotypeToImage(state,
                            state.get('backAst'),
                            genotype,
                            imageElement);
      i++;
      setTimeout(go);
    }
  });
}

function showPlaceholderImages(state) {
  const placeholder = state.get('placeholder');
  const populationSize = state.get('populationSize');
  const phenotypes = state.get('phenotypes');
  for (let i = 0; i < populationSize; i++) {
    const imageElement = phenotypes.getIn([i, 'imageElement']);
    imageElement.src = placeholder;
  }
}

// update the selected phenotypes in the evolve screen according to the
// values in selectedIndices
function updateSelectionUI(state) {

  const selectedIndices = state.getIn(['saveState', 'selectedIndices']);

  // clean up the dom and clear the selected state
  const populationSize = state.get('populationSize');
  const phenotypes = state.get('phenotypes');
  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes.getIn([i, 'phenotypeElement']);
    element.classList.remove('selected');
  }

  selectedIndices.forEach(i => {
    const element = state.getIn(['phenotypes', i, 'phenotypeElement']);
    element.classList.add('selected');
    return true;
  });
}

function onNextGen(store) {
  // get the selected genotypes for the next generation
  const populationSize = store.getState().get('populationSize');
  let selectedIndices = new Immutable.List();
  const phenotypes = store.getState().get('phenotypes');

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
  History.replaceState(store.getState().get('saveState'));

  showPlaceholderImages(store.getState());

  if (selectedIndices.size === 0) {
    // if this is the first generation and nothing has been selected
    // just randomize all of the phenotypes
    store.dispatch({type: 'INITIAL_GENERATION'});
  } else {
    const pg = store.getState().getIn(['saveState', 'genotypes']);
    let selectedGenotypes = new Immutable.List();
    for (let i = 0; i < selectedIndices.size; i++) {
      selectedGenotypes =
        selectedGenotypes.push(pg.get(selectedIndices.get(i)));
    }
    store.dispatch({type: 'NEXT_GENERATION',
                    genotypes: selectedGenotypes,
                    rng: 42});
  }

  const genotypes = store.getState().getIn(['saveState', 'genotypes']);

  // render the genotypes
  renderPhenotypes(store.getState());

  // this is the first selectedIndices.size genotypes
  const previouslySelectedGenotypes = genotypes.slice(0, selectedIndices.size);
  store.dispatch({type: 'SET_PREVIOUSLY_SELECTED_GENOTYPES',
                  previouslySelectedGenotypes});

  // clean up the dom and clear the selected state
  store.dispatch({type: 'SET_SELECTED_INDICES'});
  updateSelectionUI(store.getState());

  History.pushState(store.getState().get('saveState'));
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
        <a href="#" class="render">Render</a>
        <a href="#" class="edit">Edit</a>
      </div>`;
  return container;
}

// invoked when the evolve screen is displayed after the edit screen
function setupEvolveUI(store) {
  return new Promise((resolve, _) => {
    afterLoadingPlaceholderImages(store.getState()).then(() => {

      store.dispatch({type: 'SETUP_AST_AND_TRAITS'});
      store.dispatch({type: 'INITIAL_GENERATION'});

      // render the phenotypes
      renderPhenotypes(store.getState());
      updateSelectionUI(store.getState());

      resolve();
    });
  });
}

// invoked when restoring the evolve screen from the history api
function restoreEvolveUI(store) {
  return new Promise((resolve, _) => { // todo: implement reject
    afterLoadingPlaceholderImages(store.getState()).then(() => {
      store.dispatch({type: 'SETUP_AST_AND_TRAITS'});
      // render the phenotypes
      renderPhenotypes(store.getState());
      updateSelectionUI(store.getState());

      resolve();
    });
  });
}

// needs the store since imageLoadHandler rebinds store.getState()
// on every image load
//
function afterLoadingPlaceholderImages(state) {

  const allImagesLoadedSince = timeStamp => {
    const phenotypes = state.get('phenotypes');

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
  const currentMode = state.getIn(['saveState', 'currentMode']);
  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    containers.get(i).className = i === currentMode ? '' : 'hidden';
  }
  showButtonsFor(currentMode);
}

function showScriptInEditor(state) {
  const editor = gUI.editor;
  editor.getDoc().setValue(state.getIn(['saveState', 'script']));
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

  return new Promise((resolve, _reject) => {
    const [index, _] = getGalleryItemIdFromDom(element);
    if (index !== -1) {
      const url = `/gallery/${index}`;

      get(url).catch(() => {
        console.error(`cannot connect to ${url}`);
      }).then(data => {
        store.dispatch({type: 'SET_SCRIPT', script: data});
        return ensureMode(store, SeniMode.edit);
      }).then(() => {
        resolve();
      });
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


function createKonsole(store, element) {

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
  addDefaultCommands(store, commander);

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
    konsole: createKonsole(store, konsoleElement),
    editor: createEditor(store, editorTextArea)
  };

  konsoleElement.style.height = `0%`;

  showButtonsFor(SeniMode.gallery);

  const galleryModeHandler = event => {
    ensureMode(store, SeniMode.gallery);
    event.preventDefault();
  };

  const evolveModeHandler = event => {
    // get the latest script from the editor
    store.dispatch({type: 'SET_SCRIPT', script: getScriptFromEditor()});
    History.replaceState(store.getState().get('saveState'));

    ensureMode(store, SeniMode.evolve);
    event.preventDefault();
  };

  addClickEvent('home', galleryModeHandler);
  addClickEvent('evolve-btn', evolveModeHandler);

  addClickEvent('shuffle-btn', event => {

    showPlaceholderImages(store.getState());

    const previouslySelectedGenotypes = store.getState().
            getIn(['saveState', 'previouslySelectedGenotypes']);
    store.dispatch({type: 'NEXT_GENERATION',
                    genotypes: previouslySelectedGenotypes,
                    rng: 11});

    renderPhenotypes(store.getState());

    // clean up the dom and clear the selected state
    store.dispatch({type: 'SET_SELECTED_INDICES'});
    updateSelectionUI(store.getState());


//    store = genotypesFromSelectedPhenotypes(store);
    event.preventDefault();
  });

  addClickEvent('eval-btn', () => {
    store.dispatch({type: 'SET_SCRIPT', script: getScriptFromEditor()});
    timedRenderScript(store.getState());
  });

  addClickEvent('gallery-container', event => {
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      showEditFromGallery(store, target);
    }
    event.preventDefault();
  });

  addClickEvent('evolve-container', event => {
    const target = event.target;
    if (target.classList.contains('render')) {
      renderHighRes(store.getState(), target);
    } else if (target.classList.contains('edit')) {
      showEditFromEvolve(store, target);
    } else {
      const [index, e] = getPhenoIdFromDom(target);
      if (index !== -1) {
        e.classList.toggle('selected');
      }
    }
    event.preventDefault();
  });

  addClickEvent('next-btn', () => {
    onNextGen(store);
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
        store.getState().getIn(['saveState', 'currentMode']) ===
        SeniMode.evolve) {
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

  store.dispatch({type: 'SET_PHENOTYPES',
                  phenotypes: new Immutable.List(phenotypes)});

  window.addEventListener('popstate', event => {
    if (event.state) {
      const saveState = History.restoreState(event.state);
      store.dispatch({type: 'SET_SAVE_STATE', saveState});
      updateUI(store.getState());
      if (store.getState().getIn(['saveState', 'currentMode']) ===
          SeniMode.evolve) {
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

    konsoleToggle = 1 - konsoleToggle;
    if (konsoleToggle === 1) {
      konsolePanel.style.height = '50%';
    } else {
      konsolePanel.style.height = '0%';
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
  const state = createInitialState(gRenderer);
  const store = createStore(state);
  setupUI(store);

  getGallery()
    .then(removeKonsoleInvisibility)
    .catch(error => console.error(error));
}
