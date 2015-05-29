/*
 *  Seni
 *  Copyright (C) 2015  Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
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

function getData(url, fn) {
  var request = new XMLHttpRequest();
  request.open('GET', url, true);

  request.onload = function() {
    if (request.status >= 200 && request.status < 400) {
      fn(request.responseText);
    } else {
      // server returned an error
    }
  };

  request.onerror = function() {
    // connection error
  };

  request.send();
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

function renderGenotypeToImage(seniApp, ast, genotype, imageElement,
                               width, height) {

  const renderer = seniApp.renderer;

  if(width !== undefined && height !== undefined) {
    renderer.preDrawScene(width, height);
  } else {
    renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);
  }

  Runtime.evalAst(seniApp.env, ast, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
}

function renderScript(seniApp, form) {
  const imageElement = seniApp.renderImage;

  const ast = Runtime.buildAst(seniApp.env, form);

  const traits = Genetic.buildTraits(ast);
  const genotype = Genetic.createGenotypeFromInitialValues(traits);
  renderGenotypeToImage(seniApp, ast, genotype, imageElement);
}

function timedRenderScript(seniApp, form, msg) {
  Util.withTiming(msg, () => renderScript(seniApp, form));
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

function getGalleryItemIdFromDom(element) {
  while(element) {
    const m = element.id.match(/gallery-item-(\d+)/);
    if(m && m.length === 2) {
      const index = Number.parseInt(m[1], 10);
      return [index, element];
    } else {
      element = element.parentNode;
    }
  }
  return [-1, null];
}

/* eslint-disable no-unused-vars */
function renderHighRes(seniApp, element) {
  const [index, _] = getPhenoIdFromDom(element);
  if(index !== -1) {
    const dimmer = document.getElementById('dimmer');
    dimmer.classList.toggle('hidden');

    let piece = seniApp.piece;
    let genotype = piece.genotypes[index];
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.toggle('invisible');
    const ast = Runtime.buildAst(seniApp.env, piece.form);

    const imageElement = document.getElementById('high-res-image');
    const [width, height] = seniApp.highResolution;
    renderGenotypeToImage(seniApp, ast, genotype, imageElement,
                          width, height);

    const holder = document.getElementById('holder');
    imageElement.style.height = holder.clientHeight + 'px';
    imageElement.style.width = holder.clientWidth + 'px';

    const linkElement = document.getElementById('high-res-link');
    linkElement.href = imageElement.src;
  }
}
/* eslint-enable no-unused-vars */

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

      renderGenotypeToImage(seniApp, piece.ast, genotype, imageElement);

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

function onNextGen(seniApp) {

  // get the selected genotypes for the next generation
  let chosen = [];
  const piece = seniApp.piece;
  for(let i = 0; i < seniApp.populationSize; i++) {
    if(piece.phenotypes[i].selected === true) {
      chosen.push(piece.genotypes[i]);
    }
  }

  if(chosen.length === 0) {
    // no phenotypes were selected
    return;
  }

  showPlaceholderImages(seniApp);

  piece.genotypes = Genetic.nextGeneration(chosen, seniApp.populationSize);

  // render the genotypes
  renderPhenotypes(seniApp);

  // clean up the dom and clear the selected state
  for(let i = 0; i < seniApp.populationSize; i++) {
    if(piece.phenotypes[i].selected === true) {
      const element = piece.phenotypes[i].phenotypeElement;
      const cardImage = element.getElementsByClassName('card-image')[0];
      cardImage.classList.toggle('selected');
    }
    piece.phenotypes[i].selected = false;
  }
}

function createPhenotypeElement(id, placeholderImage) {
  const container = document.createElement('div');

  container.className = 'col s6 m4 l3';
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

function setupEvolveUI(seniApp, form) {

  showPlaceholderImages(seniApp);

  const piece = seniApp.piece;

  piece.ast = Runtime.buildAst(seniApp.env, form);
  piece.traits = Genetic.buildTraits(piece.ast);

  // create phenotype/genotype containers
  let i;

  // add genotypes to the containers
  let genotype;
  let random = new Date();
  random = random.toGMTString();
  for(i = 0; i < seniApp.populationSize; i++) {
    if(i === 0) {
      genotype = Genetic.createGenotypeFromInitialValues(piece.traits);
    } else {
      genotype = Genetic.createGenotypeFromTraits(piece.traits, i + random);
    }
    piece.genotypes[i] = genotype;
  }

  // render the phenotypes
  renderPhenotypes(seniApp);
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

    timedRenderScript(seniApp, seniApp.piece.form, 'renderScript');
    break;
  case SeniMode.evolve :
    removeNavbarClass(seniApp, 'to-gallery', 'hidden');
    removeNavbarClass(seniApp, 'to-edit', 'hidden');
    removeNavbarClass(seniApp, 'to-evolve', 'hidden');

    setupEvolveUI(seniApp, seniApp.piece.form);
    break;
  }
}
/* eslint-disable no-unused-vars */
function showFormInEditor(seniApp) {
  const editor = seniApp.editor;
  editor.getDoc().setValue(seniApp.piece.form);
  editor.refresh();
}


function showEditFromGallery(seniApp, element) {
  const [index, _] = getGalleryItemIdFromDom(element);
  if(index !== -1) {
    const url = '/gallery/' + index;
    getData(url, data => {
      // todo: construct a new piece object
      seniApp.piece.form = data;

      switchMode(seniApp, SeniMode.edit);
      showFormInEditor(seniApp);
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
  document.seni.help = function(name) {
    const v = seniApp.env.get(name);
    if(v.pb) {
      const binding = v.pb;       // publicBinding
      const args = JSON.stringify(binding.defaults, null, ' ');
      console.log(name + ':', binding.doc);
      console.log('default arguments', args);
    }
  };
  document.seni.seniApp = seniApp;
}
/*
function iterateEnv(seniApp) {
  let env = seniApp.env;
  let keys = env.keys();

  let res = [];
  for(let k = keys.next(); k.done === false; k = keys.next()) {
    res.push(k.value);
  }
  res.sort();
  res.map(name => console.log(name));
  console.log(res.toString());
}*/

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
      seniApp.piece.form = seniApp.editor.getValue();
      timedRenderScript(seniApp, seniApp.piece.form, 'renderScript');
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

  addClickEvent('action-eval', () => {
    seniApp.piece.form = seniApp.editor.getValue();
    timedRenderScript(seniApp, seniApp.piece.form, 'renderScript');
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
    } else {
      toggleSelection(seniApp, target);
    }
    event.preventDefault();
  });

  addClickEvent('action-next-gen', () => {
    onNextGen(seniApp);
  });

  addClickEvent('high-res-close', (event) => {
    console.log('high-res-close clicked');
    const highResContainer = document.getElementById('high-res-container');
    highResContainer.classList.toggle('invisible');

    const dimmer = document.getElementById('dimmer');
    dimmer.classList.toggle('hidden');
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

  const gallery = document.getElementById('phenotype-gallery');
  gallery.innerHTML = '';

  let i;
  let phenotypeElement;
  const piece = seniApp.piece;
  piece.phenotypes = [];

  let row;

  row = document.createElement('div');
  row.className = 'row';
  gallery.appendChild(row);

  for(i = 0; i < seniApp.populationSize; i++) {
    phenotypeElement = createPhenotypeElement(i, seniApp.placeholder);
    row.appendChild(phenotypeElement);

    const imageElement =
            phenotypeElement.getElementsByClassName('phenotype')[0];

    piece.phenotypes.push({
      phenotypeElement,
      imageElement,
      selected: false
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
        <a href="#" class="card-image">
          <img class="gallery-item-image show-edit"
               src="${galleryItem.image}"
               style="width:320px;height:320px">
        </a>
        <div class="card-action">
          <a href="#" class="show-edit">Edit</a>
          <a href="#" class="show-evolve">Evolve</a>
        </div>
      </div>
      `;

    return container;
  };

  getData('/gallery', data => {
    let galleryItems = JSON.parse(data);
    // gets an array of gallery items
    galleryItems.forEach(item => {
      let e = createGalleryElement(item);
      row.appendChild(e);
    });
  });
}

class Piece {
  constructor() {
    this.phenotypes = [];
    this.form = undefined;
    this.ast = undefined;
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
