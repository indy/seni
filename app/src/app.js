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

function getScriptFromEditor(app) {
  const editor = app.get('editor');
  return app.set('script', editor.getValue());
}

function ensureMode(appAtom, mode) {
  let app = appAtom.app;
  if (app.get('currentMode') === mode) {
    return appAtom;
  }

  app = app.set('currentMode', mode);

  historyPushState(app);

  appAtom.app = app;

  if (mode === SeniMode.evolve) {
    showCurrentMode(app);
    showTopNavBar(app);
    appAtom.app = getScriptFromEditor(app);
    // if it's a change of mode into the SeniMode.evolve
    appAtom = setupEvolveUI(appAtom);
    // else if there's been a change in selection ???
  } else {
    appAtom.app = updateUI(appAtom.app);
  }

  return appAtom;
}

// function that takes a read-only app and updates the UI
//
function updateUI(app) {
  showCurrentMode(app);

  switch (app.get('currentMode')) {
  case SeniMode.gallery :
    hideTopNavBar(app);
    break;
  case SeniMode.edit :
    showTopNavBar(app);
    showScriptInEditor(app);
    timedRenderScript(app, 'renderScript');
    break;
  case SeniMode.evolve :
    showTopNavBar(app);
    showPlaceholderImages(app);
    app = setupAstAndTraits(app);
    renderPhenotypes(app);
    app = updateSelectionUI(app);
    break;
  }

  return app;
}

// search the children of app.navbar for elements with class 'klass'
// then add 'addClass' to them
function addNavbarClass(app, klass, addClass) {
  const navbar = app.get('navbar');
  const es = navbar.getElementsByClassName(klass);
  for (let i = 0; i < es.length; i++) {
    es[i].classList.add(addClass);
  }
}

function removeNavbarClass(app, klass, removeClass) {
  const navbar = app.get('navbar');
  const es = navbar.getElementsByClassName(klass);
  for (let i = 0; i < es.length; i++) {
    es[i].classList.remove(removeClass);
  }
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

  const script = app.get('script');
  const frontAst = Runtime.buildFrontAst(script);
  const backAst = Runtime.compileBackAst(frontAst);
  const traits = Genetic.buildTraits(backAst);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);

  renderGenotypeToImage(app, backAst, genotype, imageElement);
}

function timedRenderScript(app, msg) {
  Util.withTiming(msg, () => renderScript(app), false);
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

function renderHighRes(app, element) {
  /* eslint-disable no-unused-vars */
  const [index, _] = getPhenoIdFromDom(element);
  /* eslint-enable no-unused-vars */

  if (index !== -1) {
    const genotypes = app.get('genotypes');
    const genotype = genotypes.get(index);
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.remove('invisible');
    const script = app.get('script');
    const frontAst = Runtime.buildFrontAst(script);
    const backAst = Runtime.compileBackAst(frontAst);

    const imageElement = document.getElementById('high-res-image');
    const [width, height] = app.get('highResolution');
    renderGenotypeToImage(app, backAst, genotype, imageElement,
                          width, height);

    const holder = document.getElementById('holder');
    imageElement.style.height = `${holder.clientHeight}px`;
    imageElement.style.width = `${holder.clientWidth}px`;

    const linkElement = document.getElementById('high-res-link');
    linkElement.href = imageElement.src;
  }
}

function showEditFromEvolve(appAtom, element) {

  const app = appAtom.app;
  /* eslint-disable no-unused-vars */
  const [index, _] = getPhenoIdFromDom(element);
  /* eslint-enable no-unused-vars */

  if (index !== -1) {
    const genotypes = app.get('genotypes');
    const genotype = genotypes.get(index);
    const frontAst = app.get('frontAst');

    const script = Runtime.unparse(frontAst, genotype);

    appAtom.app = app.set('script', script);

    appAtom = ensureMode(appAtom, SeniMode.edit);
  }

  return appAtom;
}

function toggleSelection(app, element) {
  const [index, e] = getPhenoIdFromDom(element);
  if (index !== -1) {
    const cardImage = e.getElementsByClassName('card-image')[0];
    cardImage.classList.toggle('selected');

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
    const genotypes = app.get('genotypes');
    if (i < phenotypes.size &&
        app.get('currentMode') === SeniMode.evolve) {

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

  app = app.set('genotypes', new Immutable.List(genotypes));

  return app;
}

function genotypesFromSelectedPhenotypes(app) {

  showPlaceholderImages(app);

  const selectedIndices = app.get('selectedIndices');

  if (selectedIndices.size === 0) {
    // if this is the first generation and nothing has been selected
    // just randomize all of the phenotypes
    app = createInitialGenotypePopulation(app, app.populationSize);
  } else {

    const psg = app.get('selectedIndices');
    const pg = app.get('genotypes');
    let selectedGenotypes = new Immutable.List();
    for (let i = 0; i < psg.size; i++) {
      selectedGenotypes = selectedGenotypes.push(pg.get(psg.get(i)));
    }

    const genotypes = Genetic.nextGeneration(
      selectedGenotypes,
      app.get('populationSize'),
      app.get('mutationRate'),
      app.get('traits'));
    app = app.set('genotypes', genotypes);

  }

  // render the genotypes
  renderPhenotypes(app);

  // clean up the dom and clear the selected state
  app = app.set('selectedIndices', new Immutable.List());
  app = updateSelectionUI(app);

  historyPushState(app);

  return app;
}

// update the selected phenotypes in the evolve screen according to the
// values in selectedIndices
function updateSelectionUI(app) {
  // clean up the dom and clear the selected state
  const populationSize = app.get('populationSize');
  const phenotypes = app.get('phenotypes');
  for (let i = 0; i < populationSize; i++) {
    if (phenotypes.getIn([i, 'selected']) === true) {
      const element = phenotypes.getIn([i, 'phenotypeElement']);
      const cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.remove('selected');
    }
    app = app.setIn(['phenotypes', i, 'selected'], false);
  }

  const selectedIndices = app.get('selectedIndices');
  selectedIndices.forEach(i => {
    app = app.setIn(['phenotypes', i, 'selected'], true);

    const element = app.getIn(['phenotypes', i, 'phenotypeElement']);
    const cardImage = element.getElementsByClassName('card-image')[0];
    cardImage.classList.add('selected');

    return true;
  });

  return app;
}

function onNextGen(app) {
  // get the selected genotypes for the next generation

  const populationSize = app.get('populationSize');
  let selectedIndices = new Immutable.List();
  const phenotypes = app.get('phenotypes');

  for (let i = 0; i < populationSize; i++) {
    if (phenotypes.getIn([i, 'selected']) === true) {
      selectedIndices = selectedIndices.push(i);
    }
  }

  app = app.set('selectedIndices', selectedIndices);

  if (selectedIndices.size === 0) {
    // no phenotypes were selected
    return app;
  }

  // update the last history state
  historyReplaceState(app);

  app = genotypesFromSelectedPhenotypes(app);

  return app;
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

function setupAstAndTraits(app) {
  const script = app.get('script');
  app = app.set('frontAst', Runtime.buildFrontAst(script));

  const frontAst = app.get('frontAst');
  app = app.set('backAst', Runtime.compileBackAst(frontAst));

  const backAst = app.get('backAst');
  app = app.set('traits', Genetic.buildTraits(backAst));

  return app;
}

function setupEvolveUI(appAtom) {

  let app = appAtom.app;

  const allImagesLoadedSince = function(timeStamp) {
    const app = appAtom.app;
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

  showPlaceholderImages(app);

  setTimeout(function go() {
    // wait until all of the placeholder load events have been received
    // otherwise there may be image sizing issues, especially with the
    // first img element
    if (allImagesLoadedSince(initialTimeStamp)) {

      app = setupAstAndTraits(app);

      const populationSize = app.get('populationSize');
      app = createInitialGenotypePopulation(app, populationSize);

      // update the last history state
      historyReplaceState(app);

      // render the phenotypes
      renderPhenotypes(app);

      // the reason for passing appAtom into setupEvolveUI is so
      // that it references the correct app after the conditions in this
      // timeout have been met
      appAtom.app = app;

    } else {
      setTimeout(go, 20);
    }
  });
  return appAtom;
}

function hideTopNavBar(app) {
  addNavbarClass(app, 'to-gallery', 'hidden');
  addNavbarClass(app, 'to-edit', 'hidden');
  addNavbarClass(app, 'to-evolve', 'hidden');
}

function showTopNavBar(app) {
  removeNavbarClass(app, 'to-gallery', 'hidden');
  removeNavbarClass(app, 'to-edit', 'hidden');
  removeNavbarClass(app, 'to-evolve', 'hidden');
}


function showCurrentMode(app) {
  // show the current container, hide the others
  const containers = app.get('containers');
  const currentMode = app.get('currentMode');
  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    containers.get(i).className = i === currentMode ? '' : 'hidden';
  }
}

function showScriptInEditor(app) {
  const editor = app.get('editor');
  editor.getDoc().setValue(app.get('script'));
  editor.refresh();
}

function showEditFromGallery(appAtom, element) {

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
      let sa = appAtom.app;
      sa = sa.set('script', data);
      appAtom.app = sa;
      appAtom = ensureMode(appAtom, SeniMode.edit);
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

function polluteGlobalDocument(appAtom) {
  const app = appAtom.app;

  document.seni = {};
  document.seni.title = Trivia.getTitle;
  document.seni.help = function(name, showDefaultArgs = false) {
    const v = app.getIn(['env', name]);
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
    const env = app.get('env');
    const keys = env.keys();

    const res = [];
    for (let k = keys.next(); k.done === false; k = keys.next()) {
      res.push(k.value);
    }
    res.sort();
    res.map(name => console.log(name));
  };
  document.seni.app = app;
}

function setupUI(appAtom) {
  let sa = appAtom.app;
  const d = document;

  sa = sa
    .set('navbar', document.getElementById('seni-navbar'))
    .set('renderImage', document.getElementById('render-img'))
    .set('containers',
         new Immutable.List([document.getElementById('gallery-container'),
                             document.getElementById('edit-container'),
                             document.getElementById('evolve-container')]));

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
      appAtom.app = getScriptFromEditor(appAtom.app);
      timedRenderScript(appAtom.app, 'renderScript');
      return false;
    },
    'Ctrl-D': () => false,
    'Ctrl-I': () => {
      const editor = appAtom.app.get('editor');
      const numLines = editor.doc.size;
      blockIndent(editor, 0, numLines);
      console.log('indenting', numLines, 'lines');
      return false;
    }
  };

  const textArea = d.getElementById('codemirror-textarea');
  sa = sa.set('editor', codeMirror.fromTextArea(textArea, config));

  const galleryModeHandler = event => {
    appAtom = ensureMode(appAtom, SeniMode.gallery);
    event.preventDefault();
  };

  const evolveModeHandler = event => {
    appAtom = ensureMode(appAtom, SeniMode.evolve);
    event.preventDefault();
  };

  const editModeHandler = event => {
    appAtom = ensureMode(appAtom, SeniMode.edit);
    event.preventDefault();
  };


  addClickEvent('evolve-mode-icon', evolveModeHandler);
  addClickEventForClass('to-evolve', evolveModeHandler);

  addClickEvent('edit-mode-icon', editModeHandler);
  addClickEventForClass('to-edit', editModeHandler);

  addClickEventForClass('to-gallery', galleryModeHandler);

  addClickEvent('shuffle-icon', event => {
    const app = appAtom.app;
    appAtom.app = genotypesFromSelectedPhenotypes(app);
    event.preventDefault();
  });

  addClickEvent('action-eval', () => {
    let app = appAtom.app;
    const editor = app.get('editor');
    app = app.set('script', editor.getValue());
    timedRenderScript(app, 'renderScript');
    appAtom.app = app;
  });

  addClickEvent('action-add', () => {
    const app = appAtom.app;
    appAtom.app = app.set('script', '');
    appAtom = ensureMode(appAtom, SeniMode.edit);
  });

  addClickEvent('gallery-list', event => {
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      showEditFromGallery(appAtom, target);
    }
    event.preventDefault();
  });

  addClickEvent('phenotype-gallery', event => {
    const app = appAtom.app;

    const target = event.target;
    if (target.classList.contains('render')) {
      renderHighRes(app, target);
    } else if (target.classList.contains('edit')) {
      appAtom = showEditFromEvolve(appAtom, target);
    } else {
      appAtom.app = toggleSelection(app, target);
    }
    event.preventDefault();
  });

  addClickEvent('action-next-gen', () => {
    appAtom.app = onNextGen(appAtom.app);
  });

  addClickEvent('high-res-close', event => {
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.add('invisible');

    event.preventDefault();
  });

  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    const app = appAtom.app;
    if (event.ctrlKey && event.keyCode === dKey &&
        app.get('currentMode') === SeniMode.evolve) {
      event.preventDefault();
      appAtom.app = onNextGen(app);
    }
  }, false);

  // invoked on every load event for an img tag
  const imageLoadHandler = event => {
    const app = appAtom.app;
    const imageId = event.target.getAttribute('data-id');
    appAtom.app = app.setIn(['phenotypes', imageId, 'imageLoadTimeStamp'],
                            event.timeStamp);
  };

  const gallery = document.getElementById('phenotype-gallery');
  gallery.innerHTML = '';

  let phenotypeElement, imageElement;
  // sa = sa.set('phenotypes', []);

  const row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

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
    appAtom.app = historyRestoreState(appAtom.app, event.state);
  });

  appAtom.app = sa;

  return appAtom;
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

let jjj = 1;
function historyBuildState(app) {
  // can't store the entire app since it contains DOM elements and there
  // is a 640k size limit on the serialized data structures.
  //
  const state = {
    stateCounter: jjj,
    currentMode: app.get('currentMode'),
    selectedIndices: app.get('selectedIndices').toJS(),
    script: app.get('script'),
    genotypes: app.get('genotypes').toJS()
  };

  const uri = `#${seniModeAsString(app.get('currentMode'))}-${jjj}`;
  jjj += 1;
  return [state, uri];
}

function historyPushState(app) {
  const [state, uri] = historyBuildState(app);
  // console.log('historyPushState', state);
  history.pushState(state, null, uri);
}

function historyReplaceState(app) {
  const [state, uri] = historyBuildState(app);
  // console.log('historyReplace', state);
  history.replaceState(state, null, uri);
}

function historyRestoreState(app, state) {
  // console.log('historyRestore', state);

  /**
   * Note: would like to use:
   *
   *    app = app.merge(state);
   *
   * but some of the genotypes may contain values that are plain JS arrays
   * e.g. seni code like:
   *
   * (define coords {[[10 10] [20 20] [20 20]] (vector)})
   *
   * calling merge will convert them into Immutable objects and that will
   * screw up the later stages that expect plain JS objects/primitives
   */

  const genotypes = state.genotypes.reduce((list, genotype) => {
    const gt = genotype.reduce((lst, g) => lst.push(g), new Immutable.List());
    return list.push(gt);
  }, new Immutable.List());

  app = app
    .set('currentMode', state.currentMode)
    .set('selectedIndices', new Immutable.List(state.selectedIndices))
    .set('script', state.script)
    .set('genotypes', genotypes);

  app = updateUI(app);
  return app;
}

/**
 * Creates the immutable SeniApp
 *
 * @private
 * @returns {Immutable Map} a basic SeniApp with a valid renderer and env
 */
function createSeniApp() {
  let app = Immutable.fromJS({
    currentMode: SeniMode.gallery,
    renderer: undefined,
    editor: undefined,
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
    // selectedGenotypes is required to remember the previous selection
    // in case of a shuffle
    selectedIndices: [],
    script: undefined,
    frontAst: undefined,
    backAst: undefined,
    traits: undefined,
    genotypes: [],

    // for browser history modification
    lastState: undefined
  });

  const canvasElement = document.getElementById('render-canvas');
  const renderer = new Renderer(canvasElement);
  const bindings = Bind.addBindings(Runtime.createEnv(), renderer);

  app = app
    .set('renderer', renderer)
    .set('env', bindings);

  historyPushState(app);

  return {app};
}

const SeniWebApplication = {
  mainFn() {
    resizeContainers();

    let appAtom = createSeniApp();

    polluteGlobalDocument(appAtom);
    appAtom = setupUI(appAtom);
    getGallery();
  }
};

export default SeniWebApplication;
