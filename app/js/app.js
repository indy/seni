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

import Renderer from './seni/Renderer';
import Bind from './seni/Bind';
import Util from './seni/Util';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';
import { SeniMode } from './ui/SeniMode';
import Konsole from './ui/Konsole';
import Editor from './ui/Editor';
import KonsoleCommander from './ui/KonsoleCommander';
import { addDefaultCommands } from './ui/KonsoleCommands';
import History from './ui/History';

import Immutable from 'immutable';

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

function getScriptFromEditor(app) {
  const editor = app.get('editor');
  return editor.getValue();
}

function showButtonsFor(mode) {
  const evalBtn = document.getElementById('eval-btn');
  const evolveBtn = document.getElementById('evolve-btn');
  const nextBtn = document.getElementById('next-btn');
  const shuffleBtn = document.getElementById('shuffle-btn');

  switch (mode) {
  case SeniMode.gallery :
    evalBtn.classList.add('inactive-button');
    evolveBtn.classList.add('inactive-button');
    nextBtn.classList.add('inactive-button');
    shuffleBtn.classList.add('inactive-button');
    break;
  case SeniMode.edit :
    evalBtn.classList.remove('inactive-button');
    evolveBtn.classList.remove('inactive-button');
    nextBtn.classList.add('inactive-button');
    shuffleBtn.classList.add('inactive-button');
    break;
  case SeniMode.evolve :
    evalBtn.classList.add('inactive-button');
    evolveBtn.classList.add('inactive-button');
    nextBtn.classList.remove('inactive-button');
    shuffleBtn.classList.remove('inactive-button');
    break;
  }
}

function ensureMode(atom, mode) {

  if (atom.app.getIn(['appState', 'currentMode']) === mode) {
    return atom;
  }

  atom.app = atom.app.setIn(['appState', 'currentMode'], mode);

  if (mode === SeniMode.evolve) {
    showCurrentMode(atom.app);
    atom = setupEvolveUI(atom);
  } else {
    atom = updateUI(atom);
  }

  History.pushState(atom.app.get('appState'));

  return atom;
}

// function that takes a read-only app and updates the UI
//
function updateUI(atom) {
  showCurrentMode(atom.app);

  switch (atom.app.getIn(['appState', 'currentMode'])) {
  case SeniMode.gallery :
    break;
  case SeniMode.edit :
    showScriptInEditor(atom.app);
    timedRenderScript(atom.app);
    break;
  case SeniMode.evolve :
    // will only get called from History.restoreState
    //
    atom = restoreEvolveUI(atom);
    break;
  }

  return atom;
}

function renderGenotypeToImage(app, ast, genotype, imageElement, w, h) {

  const renderer = app.get('renderer');

  if (w !== undefined && h !== undefined) {
    renderer.preDrawScene(w, h);
  } else {
    renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);
  }

  Runtime.evalAst(app.get('env'), ast, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
}

function renderScript(app) {
  const imageElement = app.get('renderImage');

  const script = app.getIn(['appState', 'script']);
  const frontAst = Runtime.buildFrontAst(script);
  const backAst = Runtime.compileBackAst(frontAst);
  const traits = Genetic.buildTraits(backAst);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  renderGenotypeToImage(app, backAst, genotype, imageElement);
}

function timedRenderScript(app) {
  Util.withTiming('rendered', () => renderScript(app),
                  app.get('konsole'));
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

function renderHighRes(app, element) {
  console.log('renderHighRes');

  const [index, _] = getPhenoIdFromDom(element);

  if (index !== -1) {
    const genotypes = app.getIn(['appState', 'genotypes']);
    const genotype = genotypes.get(index);
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.remove('hidden');
    const script = app.getIn(['appState', 'script']);
    const frontAst = Runtime.buildFrontAst(script);
    const backAst = Runtime.compileBackAst(frontAst);

    const imageElement = document.getElementById('high-res-image');
    const [width, height] = app.get('highResolution');
    renderGenotypeToImage(app, backAst, genotype, imageElement,
                          width, height);

    const linkElement = document.getElementById('high-res-link');
    linkElement.href = imageElement.src;
  }
}

function showEditFromEvolve(atom, element) {

  const app = atom.app;
  const [index, _] = getPhenoIdFromDom(element);

  if (index !== -1) {
    const genotypes = app.getIn(['appState', 'genotypes']);
    const genotype = genotypes.get(index);
    const frontAst = app.get('frontAst');

    const script = Runtime.unparse(frontAst, genotype);

    atom.app = app.setIn(['appState', 'script'], script);

    atom = ensureMode(atom, SeniMode.edit);
  }

  return atom;
}

function toggleSelection(app, element) {
  const [index, e] = getPhenoIdFromDom(element);
  if (index !== -1) {
    e.classList.toggle('selected');

    const path = ['phenotypes', index, 'selected'];
    const selected = app.getIn(path);
    app = app.setIn(path, !selected);
  }

  return app;
}

function renderPhenotypes(app) {

  let i = 0;
  setTimeout(function go() {
    // stop generating new phenotypes if we've reached the desired
    // population or the user has switched to edit mode
    const phenotypes = app.get('phenotypes');
    const genotypes = app.getIn(['appState', 'genotypes']);
    if (i < phenotypes.size &&
        app.getIn(['appState', 'currentMode']) === SeniMode.evolve) {

      const genotype = genotypes.get(i);
      const imageElement = phenotypes.getIn([i, 'imageElement']);

      renderGenotypeToImage(app,
                            app.get('backAst'),
                            genotype,
                            imageElement);
      i++;
      setTimeout(go);
    }
  });
}

function showPlaceholderImages(app) {
  const placeholder = app.get('placeholder');
  const populationSize = app.get('populationSize');
  const phenotypes = app.get('phenotypes');
  for (let i = 0; i < populationSize; i++) {
    const imageElement = phenotypes.getIn([i, 'imageElement']);
    imageElement.src = placeholder;
  }
}

// returns an immutable list of genotypes
function createInitialGenotypePopulation(app, populationSize) {
  // add genotypes to the containers
  let genotype;
  const random = (new Date()).toGMTString();
  const traits = app.get('traits');
  const genotypes = [];

  for (let i = 0; i < populationSize; i++) {
    if (i === 0) {
      genotype = Genetic.createGenotypeFromInitialValues(traits);
    } else {
      genotype = Genetic.createGenotypeFromTraits(traits, i + random);
    }
    genotypes.push(genotype);
  }

  return new Immutable.List(genotypes);
}

// update the selected phenotypes in the evolve screen according to the
// values in selectedIndices
function updateSelectionUI(app) {

  const selectedIndices = app.getIn(['appState', 'selectedIndices']);
  let s = '';
  selectedIndices.forEach(i => {
    s = `${s}, ${i}`;
  });
  console.log('updateSelectionUI selectedIndices:', s);

  // clean up the dom and clear the selected state
  const populationSize = app.get('populationSize');
  const phenotypes = app.get('phenotypes');
  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes.getIn([i, 'phenotypeElement']);
    element.classList.remove('selected');
    app = app.setIn(['phenotypes', i, 'selected'], false);
  }

  selectedIndices.forEach(i => {
    app = app.setIn(['phenotypes', i, 'selected'], true);

    const element = app.getIn(['phenotypes', i, 'phenotypeElement']);
    element.classList.add('selected');

    return true;
  });

  return app;
}

function onNextGen(atom) {
  let app = atom.app;

  // get the selected genotypes for the next generation
  const populationSize = app.get('populationSize');
  let selectedIndices = new Immutable.List();
  const phenotypes = app.get('phenotypes');

  for (let i = 0; i < populationSize; i++) {
    if (phenotypes.getIn([i, 'selected']) === true) {
      selectedIndices = selectedIndices.push(i);
    }
  }

  app = app.setIn(['appState', 'selectedIndices'], selectedIndices);

  if (selectedIndices.size === 0) {
    // no phenotypes were selected
    atom.app = app;
    return atom;
  }

  // update the last history state
  History.replaceState(app.get('appState'));

  showPlaceholderImages(app);

  let genotypes;

  if (selectedIndices.size === 0) {
    // if this is the first generation and nothing has been selected
    // just randomize all of the phenotypes
    genotypes = createInitialGenotypePopulation(app, app.get('populationSize'));
  } else {
    const pg = app.getIn(['appState', 'genotypes']);
    let selectedGenotypes = new Immutable.List();
    for (let i = 0; i < selectedIndices.size; i++) {
      selectedGenotypes =
        selectedGenotypes.push(pg.get(selectedIndices.get(i)));
    }

    genotypes = Genetic.nextGeneration(
      selectedGenotypes,
      app.get('populationSize'),
      app.get('mutationRate'),
      app.get('traits'), 42);
  }

  app = app.setIn(['appState', 'genotypes'], genotypes);

  // render the genotypes
  renderPhenotypes(app);

  // this is the first selectedIndices.size genotypes
  app = app.setIn(['appState', 'previouslySelectedGenotypes'],
                genotypes.slice(0, selectedIndices.size));

  // clean up the dom and clear the selected state
  app = app.setIn(['appState', 'selectedIndices'], new Immutable.List());
  app = updateSelectionUI(app);

  History.pushState(app.get('appState'));

  atom.app = app;

  return atom;
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

function setupAstAndTraits(app) {
  const script = app.getIn(['appState', 'script']);
  app = app.set('frontAst', Runtime.buildFrontAst(script));

  const frontAst = app.get('frontAst');
  app = app.set('backAst', Runtime.compileBackAst(frontAst));

  const backAst = app.get('backAst');
  app = app.set('traits', Genetic.buildTraits(backAst));

  return app;
}

// invoked when the evolve screen is displayed after the edit screen
function setupEvolveUI(atom) {
  return afterLoadingPlaceholderImages(atom, app => {
    app = setupAstAndTraits(app);
    const populationSize = app.get('populationSize');
    const genotypes = createInitialGenotypePopulation(app, populationSize);
    app = app.setIn(['appState', 'genotypes'], genotypes);
    // make sure that the history for the first evolve generation
    // has the correct genotypes
    History.replaceState(app.get('appState'));
    // render the phenotypes
    renderPhenotypes(app);
    return updateSelectionUI(app);
  });
}

// invoked when restoring the evolve screen from the history api
function restoreEvolveUI(atom) {
  return afterLoadingPlaceholderImages(atom, app => {
    app = setupAstAndTraits(app);
    // render the phenotypes
    renderPhenotypes(app);
    return updateSelectionUI(app);
  });
}

// callback accepts an app argument
function afterLoadingPlaceholderImages(atom, callback) {

  const allImagesLoadedSince = function(timeStamp) {
    const app = atom.app;
    const populationSize = app.get('populationSize');
    const phenotypes = app.get('phenotypes');
    for (let i = 0; i < populationSize; i++) {
      if (phenotypes.getIn([i, 'imageLoadTimeStamp']) < timeStamp) {
        return false;
      }
    }
    return true;
  };

  const initialTimeStamp = Date.now();

  showPlaceholderImages(atom.app);

  setTimeout(function go() {
    // wait until all of the placeholder load events have been received
    // otherwise there may be image sizing issues, especially with the
    // first img element
    if (allImagesLoadedSince(initialTimeStamp)) {
      atom.app = callback(atom.app);
    } else {
      setTimeout(go, 20);
    }
  });
  return atom;
}

function showCurrentMode(app) {
  // show the current container, hide the others
  const containers = app.get('containers');
  const currentMode = app.getIn(['appState', 'currentMode']);
  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    containers.get(i).className = i === currentMode ? '' : 'hidden';
  }
  showButtonsFor(currentMode);
}

function showScriptInEditor(app) {
  const editor = app.get('editor');
  editor.getDoc().setValue(app.getIn(['appState', 'script']));
  editor.refresh();
}

function showEditFromGallery(atom, element) {

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
      let sa = atom.app;
      sa = sa.setIn(['appState', 'script'], data);
      atom.app = sa;
      atom = ensureMode(atom, SeniMode.edit);
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

function createKonsole(atom, element) {

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
  addDefaultCommands(atom, commander);

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

function createEditor(atom, editorTextArea) {

  const blockIndent = function(editor, from, to) {
    editor.operation(() => {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  return Editor.createEditor(editorTextArea, {
    theme: 'default',
    extraKeys: {
      'Ctrl-E': () => {
        atom.app = atom.app.setIn(['appState', 'script'],
                                  getScriptFromEditor(atom.app));
        timedRenderScript(atom.app);
        return false;
      },
      // make ctrl-m a noop, otherwise invoking the konsole will result in
      // deleting a line from the editor
      'Ctrl-M': () => false,
      'Ctrl-I': () => {
        const editor = atom.app.get('editor');
        const konsole = atom.app.get('konsole');
        const numLines = editor.doc.size;
        blockIndent(editor, 0, numLines);
        konsole.log(`indenting ${numLines} lines`);
        return false;
      }
    }
  });
}

function setupUI(atom) {
  let sa = atom.app;
  const d = document;

  sa = sa
    .set('navbar', document.getElementById('seni-navbar'))
    .set('renderImage', document.getElementById('render-img'))
    .set('containers',
         new Immutable.List([document.getElementById('gallery-container'),
                             document.getElementById('edit-container'),
                             document.getElementById('evolve-container')]));

  showButtonsFor(SeniMode.gallery);

  const konsoleElement = document.getElementById('konsole');
  const konsole = createKonsole(atom, konsoleElement);
  sa = sa.set('konsole', konsole);
  konsoleElement.style.height = `0%`;

  const editorTextArea = d.getElementById('edit-textarea');
  const editor = createEditor(atom, editorTextArea);
  sa = sa.set('editor', editor);

  const galleryModeHandler = event => {
    atom = ensureMode(atom, SeniMode.gallery);
    event.preventDefault();
  };

  const evolveModeHandler = event => {
    // get the latest script from the editor
    atom.app = atom.app.setIn(['appState', 'script'],
                                    getScriptFromEditor(atom.app));
    History.replaceState(atom.app.get('appState'));

    atom = ensureMode(atom, SeniMode.evolve);
    event.preventDefault();
  };

  addClickEvent('home', galleryModeHandler);
  addClickEvent('evolve-btn', evolveModeHandler);

  addClickEvent('shuffle-btn', event => {

    let app = atom.app;

    showPlaceholderImages(app);

    app = app.setIn(['appState', 'genotypes'], Genetic.nextGeneration(
      app.getIn(['appState', 'previouslySelectedGenotypes']),
      app.get('populationSize'),
      app.get('mutationRate'),
      app.get('traits'),
      11));

    // render the genotypes
    renderPhenotypes(app);

    // clean up the dom and clear the selected state
    app = app.setIn(['appState', 'selectedIndices'], new Immutable.List());
    app = updateSelectionUI(app);

    atom.app = app;

//    atom = genotypesFromSelectedPhenotypes(atom);
    event.preventDefault();
  });

  addClickEvent('eval-btn', () => {
    let app = atom.app;
    const editor = app.get('editor');
    app = app.setIn(['appState', 'script'], editor.getValue());
    timedRenderScript(app);
    atom.app = app;
  });
/*
  addClickEvent('action-add', () => {
    const app = atom.app;
    atom.app = app.set('script', '');
    atom = ensureMode(atom, SeniMode.edit);
  });
*/
  addClickEvent('gallery-container', event => {
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      showEditFromGallery(atom, target);
    }
    event.preventDefault();
  });

  addClickEvent('evolve-container', event => {
    const app = atom.app;

    const target = event.target;
    if (target.classList.contains('render')) {
      renderHighRes(app, target);
    } else if (target.classList.contains('edit')) {
      atom = showEditFromEvolve(atom, target);
    } else {
      atom.app = toggleSelection(app, target);
    }
    event.preventDefault();
  });

  addClickEvent('next-btn', () => {
    atom = onNextGen(atom);
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
        atom.app.getIn(['appState', 'currentMode']) === SeniMode.evolve) {
      event.preventDefault();
      atom = onNextGen(atom);
    }
  }, false);

  // invoked on every load event for an img tag
  const imageLoadHandler = event => {
    const app = atom.app;
    const imageId = event.target.getAttribute('data-id');
    atom.app = app.setIn(['phenotypes', imageId, 'imageLoadTimeStamp'],
                            event.timeStamp);
  };

  // setup the evolve-container
  const evolveGallery = document.getElementById('evolve-gallery');
  evolveGallery.innerHTML = '';

  const row = document.createElement('div');
  row.className = 'cards';
  evolveGallery.appendChild(row);

  let phenotypeElement, imageElement;

  const populationSize = sa.get('populationSize');
  const phenotypes = [];
  for (let i = 0; i < populationSize; i++) {
    phenotypeElement = createPhenotypeElement(i, '');

    // get the image element
    imageElement = phenotypeElement.getElementsByClassName('phenotype')[0];
    imageElement.addEventListener('load', imageLoadHandler, false);

    row.appendChild(phenotypeElement);

    phenotypes.push(new Immutable.Map({
      phenotypeElement,
      imageElement,
      selected: false,
      imageLoadTimeStamp: 0
    }));
  }

  sa = sa.set('phenotypes', new Immutable.List(phenotypes));

  window.addEventListener('popstate', event => {
    if (event.state) {
      const appState = History.restoreState(event.state);
      atom.app = atom.app.set('appState', appState);
      atom = updateUI(atom);
      // atom = History.restoreState(atom, event.state);
    } else {
      // no event.state so behave as if the user has visited the '/' of the app
      atom = ensureMode(atom, SeniMode.gallery);
    }
  });

  atom.app = sa;

  let konsoleToggle = 0;
  document.onkeydown = evt => {
    evt = evt || window.event;

    // Ctrl-M
    if (evt.ctrlKey && evt.keyCode == 77) {
      const konsolePanel = document.getElementById('konsole');

      konsoleToggle = 1 - konsoleToggle;
      if (konsoleToggle === 1) {
        konsolePanel.style.height = '50%';
      } else {
        konsolePanel.style.height = '0%';
      }
      atom.app.get('konsole').refresh();
      atom.app.get('editor').refresh();
    }
  };

  return atom;
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

// stops the konsole from briefly flashing at app startup
// probably better to remove this and replace with some other
// sort of CSS cleverness. (resorting to this since a CSS rule
// of 'position: fixed;height:0;' for #konsole screws up Chrome
// and requires a restart)
function removeKonsoleInvisibility() {
  const k = document.getElementById('konsole');
  k.classList.remove('invisible');
}

function createAppState() {
  return Immutable.fromJS({
    currentMode: SeniMode.gallery,
    previouslySelectedGenotypes: [],
    selectedIndices: [],
    script: undefined,
    genotypes: []
  });
}

/**
 * Creates the immutable SeniApp
 *
 * @private
 * @returns {Immutable Map} a basic SeniApp with a valid renderer and env
 */
function createSeniApp() {
  let app = Immutable.fromJS({
    renderer: undefined,
    editor: undefined,

    // console CodeMirror element in the edit screen
    konsole: undefined,

    // the top nav bar across the app
    navbar: undefined,
    // the img destination that shows the rendered script in edit mode
    renderImage: undefined,
    // the resolution of the high res image
    highResolution: [2048, 2048],
    // the 3 main UI areas, stored in an Immutable.List
    containers: [],
    placeholder: 'img/spinner.gif',
    populationSize: 24,
    mutationRate: 0.1,
    // an immutable var containing the base env for all evaluations
    env: undefined,

    // information about the current piece being created/rendered
    phenotypes: [], // stored in an Immutable.List
    frontAst: undefined,
    backAst: undefined,
    traits: undefined,

    appState: createAppState()
  });

  const canvasElement = document.getElementById('render-canvas');
  const renderer = new Renderer(canvasElement);
  const bindings = Bind.addBindings(Runtime.createEnv(), renderer);

  app = app
    .set('renderer', renderer)
    .set('env', bindings);

  return {app};
}

export default function main() {

  resizeContainers();

  // creates the atom
  setupUI(createSeniApp());

  getGallery()
    .then(removeKonsoleInvisibility)
    .catch(error => console.error(error));
}
