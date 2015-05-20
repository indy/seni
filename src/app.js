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
import CodeMirrorConfig from './ui/CodeMirrorConfig';

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

function renderScriptToImage(seniApp, ast, genotype, imageElement) {

  const renderer = seniApp.renderer;

  renderer.preDrawScene(imageElement.clientWidth, imageElement.clientHeight);

  Runtime.evalAst(seniApp.env, ast, genotype);

  renderer.postDrawScene();

  imageElement.src = renderer.getImageData();
}

function renderScript(seniApp, form) {
  const imageElement = seniApp.renderImage;

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
    let piece = seniApp.piece;
    let genotype = piece.genotypes[index];
    const imageElement = document.getElementById('high-res-image');
    imageElement.classList.toggle('hidden');
    const ast = Runtime.buildAst(seniApp.env, piece.form);
    renderScriptToImage(seniApp, ast, genotype, imageElement);
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

      seniApp.renderer.preDrawScene(imageElement.clientWidth,
                                imageElement.clientHeight);

      Runtime.evalAst(seniApp.env, piece.ast, genotype);
      seniApp.renderer.postDrawScene();

      imageElement.src = seniApp.renderer.getImageData();

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
  showPlaceholderImages(seniApp);
  // get the selected genotypes for the next generation
  let chosen = [];
  const piece = seniApp.piece;
  for(let i = 0; i < seniApp.populationSize; i++) {
    if(piece.phenotypes[i].selected === true) {
      chosen.push(piece.genotypes[i]);
    }
  }

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
      seniApp.currentMode = newMode;
    break;
  case SeniMode.edit :
    renderScript(seniApp, seniApp.piece.form);
    break;
  case SeniMode.evolve :
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
    console.log('showEditFromGallery called with', index);
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

  const ac = document.getElementById('edit-container');
  ac.style.height = (ac.offsetHeight - navbar.offsetHeight) + 'px';

  const sc = document.getElementById('evolve-container');
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

  seniApp.containers = [
    document.getElementById('gallery-container'),
    document.getElementById('edit-container'),
    document.getElementById('evolve-container')
  ];

  seniApp.renderImage = document.getElementById('render-img');

  const textArea = d.getElementById('codemirror-textarea');
  //textArea.value = initialCode();

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
      withTiming('renderTime', () =>
                 renderScript(seniApp, seniApp.piece.form));
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
    withTiming('renderTime', () => renderScript(seniApp, seniApp.piece.form));
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

  addClickEvent('high-res-image', (event) => {
    event.target.classList.toggle('hidden');
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
        <div class="card-image">
          <img class="gallery-item-image"
               src="${galleryItem.image}"
               style="width:320px;height:320px">
        </div>
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
  const seniApp = {
    currentMode: SeniMode.edit,
    renderer: new Renderer('render-canvas'),
    editor: undefined,
    // the img destination that shows the rendered script in edit mode
    renderImage: undefined,
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
    seniApp.piece.form = initialCode();
    showFormInEditor(seniApp);
    renderScript(seniApp, seniApp.piece.form);

    getGallery();
    //iterateEnv(seniApp);
  }
};

export default SeniWebApplication;
