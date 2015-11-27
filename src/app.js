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

function get(url) {
  return new Promise((resolve, reject) => {

    let req = new XMLHttpRequest();
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


// search the children of seniApp.navbar for elements with class 'klass'
// then add 'addClass' to them
function addNavbarClass(seniApp, klass, addClass) {
  let es = seniApp.navbar.getElementsByClassName(klass);
  for(let i = 0; i < es.length; i++) {
    es[i].classList.add(addClass);
  }
}

function removeNavbarClass(seniApp, klass, removeClass) {
  let es = seniApp.navbar.getElementsByClassName(klass);
  for(let i = 0; i < es.length; i++) {
    es[i].classList.remove(removeClass);
  }
}

function renderGenotypeToImage(seniApp, ast, genotype, imageElement, w, h) {

  const renderer = seniApp.renderer;

  if(w !== undefined && h !== undefined) {
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
  Util.withTiming(msg, () => renderScript(seniApp));
}

function addClickEvent(id, fn) {
  const element = document.getElementById(id);
  element.addEventListener('click', fn);
}

function addClickEventForClass(className, fn) {
  const elements = document.getElementsByClassName(className);
  // getElementsByClassName returns an array-like object
  for(let i = 0; i < elements.length; i++) {
    elements[i].addEventListener('click', fn);
  }
}

// when user has clicked on a phenotype in the evolve UI,
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
  let [index, _] = getPhenoIdFromDom(element);
  _ = _;

  if(index !== -1) {
    let piece = seniApp.piece;
    let genotype = piece.genotypes[index];
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.remove('invisible');
    const frontAst = Runtime.buildFrontAst(piece.script);
    Runtime.logUnparse(frontAst, genotype);
    const backAst = Runtime.compileBackAst(frontAst);

    const imageElement = document.getElementById('high-res-image');
    const [width, height] = seniApp.highResolution;
    renderGenotypeToImage(seniApp, backAst, genotype, imageElement,
                          width, height);

    const holder = document.getElementById('holder');
    imageElement.style.height = holder.clientHeight + 'px';
    imageElement.style.width = holder.clientWidth + 'px';

    const linkElement = document.getElementById('high-res-link');
    linkElement.href = imageElement.src;
  }
}

function showEditFromEvolve(seniApp, element) {
  let [index, _] = getPhenoIdFromDom(element);
  _ = _;

  if(index !== -1) {
    const piece = seniApp.piece;
    const genotype = piece.genotypes[index];
    const frontAst = seniApp.piece.frontAst;

    const script = Runtime.unparse(frontAst, genotype);

    seniApp.piece.script = script;
    switchMode(seniApp, SeniMode.edit);
    showScriptInEditor(seniApp);
  }
}

function toggleSelection(seniApp, element) {
  const [index, e] = getPhenoIdFromDom(element);
  if(index !== -1) {
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
  for(let i = 0; i < seniApp.populationSize; i++) {
    const imageElement = piece.phenotypes[i].imageElement;
    imageElement.src = seniApp.placeholder;
  }
}

function createInitialGenotypePopulation(piece, populationSize) {
  // add genotypes to the containers
  let genotype;
  let random = new Date();
  random = random.toGMTString();
  for(let i = 0; i < populationSize; i++) {
    if(i === 0) {
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

  if(piece.selectedPhenotypes.length === 0) {
    // if this is the first generation and nothing has been selected
    // just randomize all of the phenotypes
    createInitialGenotypePopulation(piece, seniApp.populationSize);
  } else {
    piece.genotypes = Genetic.nextGeneration(piece.selectedPhenotypes,
                                             seniApp.populationSize,
                                             seniApp.mutationRate,
                                             piece.traits);
  }

  // render the genotypes
  renderPhenotypes(seniApp);

  // clean up the dom and clear the selected state
  for(let i = 0; i < seniApp.populationSize; i++) {
    if(piece.phenotypes[i].selected === true) {
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

  piece.selectedPhenotypes = [];

  for(let i = 0; i < seniApp.populationSize; i++) {
    if(piece.phenotypes[i].selected === true) {
      piece.selectedPhenotypes.push(piece.genotypes[i]);
    }
  }

  if(piece.selectedPhenotypes.length === 0) {
    // no phenotypes were selected
    return;
  }

  genotypesFromSelectedPhenotypes(seniApp);
}

function createPhenotypeElement(id, placeholderImage) {
  const container = document.createElement('div');

  container.className = 'col s6 m4 l3';
  container.id = 'pheno-' + id;

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

  getScriptFromEditor(seniApp);

  let allImagesLoadedSince = function(timeStamp) {
    let piece = seniApp.piece;
    for(let i = 0; i < seniApp.populationSize; i++) {
      if(piece.phenotypes[i].imageLoadTimeStamp < timeStamp) {
        return false;
      }
    }
    return true;
  };

  let initialTimeStamp = Date.now();

  showPlaceholderImages(seniApp);

  setTimeout(function go() {
    // wait until all of the placeholder load events have been received
    // otherwise there may be image sizing issues, especially with the
    // first img element
    if(allImagesLoadedSince(initialTimeStamp)) {
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

function switchMode(seniApp, newMode) {

  if(seniApp.currentMode === newMode) {
    return;
  }

  //const oldMode = seniApp.currentMode;
  seniApp.currentMode = newMode;

  // show the current container, hide the others
  for(let i = 0; i < SeniMode.numSeniModes; i++) {
    seniApp.containers[i].className = i === newMode ? '' : 'hidden';
  }

  switch(seniApp.currentMode) {
  case SeniMode.gallery :
    addNavbarClass(seniApp, 'to-gallery', 'hidden');
    addNavbarClass(seniApp, 'to-edit', 'hidden');
    addNavbarClass(seniApp, 'to-evolve', 'hidden');
    break;
  case SeniMode.edit :
    removeNavbarClass(seniApp, 'to-gallery', 'hidden');
    removeNavbarClass(seniApp, 'to-edit', 'hidden');
    removeNavbarClass(seniApp, 'to-evolve', 'hidden');

    timedRenderScript(seniApp, 'renderScript');
    break;
  case SeniMode.evolve :
    removeNavbarClass(seniApp, 'to-gallery', 'hidden');
    removeNavbarClass(seniApp, 'to-edit', 'hidden');
    removeNavbarClass(seniApp, 'to-evolve', 'hidden');

    setupEvolveUI(seniApp);
    break;
  }
}
/* eslint-disable no-unused-vars */


function showScriptInEditor(seniApp) {
  const editor = seniApp.editor;
  editor.getDoc().setValue(seniApp.piece.script);
  editor.refresh();
}

function showEditFromGallery(seniApp, element) {

  let getGalleryItemIdFromDom = function(e) {
    while(e) {
      const m = e.id.match(/gallery-item-(\d+)/);
      if(m && m.length === 2) {
        const idx = Number.parseInt(m[1], 10);
        return [idx, e];
      } else {
        e = e.parentNode;
      }
    }
    return [-1, null];
  };

  const [index, _] = getGalleryItemIdFromDom(element);
  if(index !== -1) {
    const url = '/gallery/' + index;

    get(url).catch(() => {
      console.error(`cannot connect to ${url}`);
    }).then((data) => {
      // todo: construct a new piece object
      seniApp.piece.script = data;

      switchMode(seniApp, SeniMode.edit);
      showScriptInEditor(seniApp);
    });
  }
}
/* eslint-enable no-unused-vars */

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  const edit = document.getElementById('edit-container');
  edit.style.height = (window.innerHeight - navbar.offsetHeight) + 'px';

  const evolve = document.getElementById('evolve-container');
  evolve.style.height = (window.innerHeight - navbar.offsetHeight) + 'px';
}

function polluteGlobalDocument(seniApp) {
  document.seni = {};
  document.seni.title = Trivia.getTitle;
  document.seni.help = function(name, showDefaultArgs = false) {
    const v = seniApp.env.get(name);
    if(v.pb) {
      const binding = v.pb;       // publicBinding
      console.log(name + ':', binding.doc);

      if(showDefaultArgs) {
        const args = JSON.stringify(binding.defaults, null, ' ');
        console.log('default arguments', args);
      }
    }
  };
  document.seni.ls = function() {
    let env = seniApp.env;
    let keys = env.keys();

    let res = [];
    for(let k = keys.next(); k.done === false; k = keys.next()) {
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

  let blockIndent = function(editor, from, to) {
    editor.operation(function() {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  let codeMirror = CodeMirrorConfig.defineSeniMode();
  let config = CodeMirrorConfig.defaultConfig;
  config.extraKeys = {
    'Ctrl-E': function() {
      getScriptFromEditor(seniApp);
      timedRenderScript(seniApp, 'renderScript');
      return false;
    },
    'Ctrl-D': function() {
      return false;
    },
    'Ctrl-I': function() {
      let numLines = seniApp.editor.doc.size;
      blockIndent(seniApp.editor, 0, numLines);
      console.log('indenting', numLines, 'lines');
      return false;
    }
  };

  const textArea = d.getElementById('codemirror-textarea');
  seniApp.editor = codeMirror.fromTextArea(textArea, config);

  let galleryModeHandler = event => {
    switchMode(seniApp, SeniMode.gallery);
    event.preventDefault();
  };

  let evolveModeHandler = event => {
    switchMode(seniApp, SeniMode.evolve);
    event.preventDefault();
  };

  let editModeHandler = event => {
    switchMode(seniApp, SeniMode.edit);
    event.preventDefault();
  };


  addClickEvent('evolve-mode-icon', evolveModeHandler);
  addClickEventForClass('to-evolve', evolveModeHandler);

  addClickEvent('edit-mode-icon', editModeHandler);
  addClickEventForClass('to-edit', editModeHandler);

  addClickEventForClass('to-gallery', galleryModeHandler);

  addClickEvent('shuffle-icon', (event) => {
    genotypesFromSelectedPhenotypes(seniApp);
    event.preventDefault();
  });

  addClickEvent('action-eval', () => {
    seniApp.piece.script = seniApp.editor.getValue();
    timedRenderScript(seniApp, 'renderScript');
  });

  addClickEvent('action-add', () => {
    seniApp.piece.script = '';
    switchMode(seniApp, SeniMode.edit);
    showScriptInEditor(seniApp);
  });

  addClickEvent('gallery-list', event => {
    let target = event.target;
    if(target.classList.contains('show-edit')) {
      showEditFromGallery(seniApp, target);
    }
    event.preventDefault();
  });

  addClickEvent('phenotype-gallery', event => {
    let target = event.target;
    if(target.classList.contains('render')) {
      renderHighRes(seniApp, target);
    } else if(target.classList.contains('edit')) {
      showEditFromEvolve(seniApp, target);
    } else {
      toggleSelection(seniApp, target);
    }
    event.preventDefault();
  });

  addClickEvent('action-next-gen', () => {
    onNextGen(seniApp);
  });

  addClickEvent('high-res-close', (event) => {
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
  let imageLoadHandler = (event) => {
    let imageId = event.target.getAttribute('data-id');
    piece.phenotypes[imageId].imageLoadTimeStamp = event.timeStamp;
  };

  const gallery = document.getElementById('phenotype-gallery');
  gallery.innerHTML = '';

  let phenotypeElement, imageElement;
  piece.phenotypes = [];

  let row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

  for(let i = 0; i < seniApp.populationSize; i++) {
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
}

function getGallery() {
  let list = document.getElementById('gallery-list');
  list.innerHTML = '';

  let row = document.createElement('div');
  row.className = 'row';
  list.appendChild(row);

  let createGalleryElement = galleryItem => {
    const container = document.createElement('div');

    container.className = 'col s6 m4 l3';
    container.id = 'gallery-item-' + galleryItem.id;

    container.innerHTML = `
      <div class="card">
        <a href="#" class="card-image show-edit">
          <img class="gallery-item-image show-edit"
               src="${galleryItem.image}"
               style="width:320px;height:320px">
        </a>
        <div class="card-action">
          <span>${galleryItem.name}</span>
        </div>
      </div>
      `;

    return container;
  };

  let url = '/gallery';
  getJSON(url).then((galleryItems) => {
    // gets an array of gallery items
    galleryItems.forEach(item => {
      let e = createGalleryElement(item);
      row.appendChild(e);
    });
  }).catch(() => {
    console.error(`cannot connect to ${url}`);
  });
}

class Piece {
  constructor() {
    this.phenotypes = [];
    // selectedPhenotypes is required to remember the previous selection
    // in case of a shuffle
    this.selectedPhenotypes = [];
    this.script = undefined;
    this.frontAst = undefined;
    this.backAst = undefined;
    this.traits = undefined;
    this.genotypes = [];
  }
}

function createSeniApp() {
  let canvasElement = document.getElementById('render-canvas');
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
    piece: new Piece()
  };

  seniApp.env = Bind.addBindings(Runtime.createEnv(), seniApp.renderer);

  return seniApp;
}

const SeniWebApplication = {
  mainFn() {
    resizeContainers();

    let seniApp = createSeniApp();

    polluteGlobalDocument(seniApp);

    setupUI(seniApp);

    getGallery();
    //iterateEnv(seniApp);
  }
};

export default SeniWebApplication;
