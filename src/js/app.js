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

import GLRenderer from './seni/GLRenderer';

import History from './ui/History';
import Editor from './ui/Editor';
import Konsole from './ui/Konsole';
//import KonsoleCommander from './ui/KonsoleCommander';
//import { addDefaultCommands } from './ui/KonsoleCommands';
import { createStore, createInitialState } from './store';
import { startTiming } from './timer';
import { SeniMode } from './ui/SeniMode';
import Job from './job';
import { jobRender,
         jobRenderWasm,
         jobUnparse
         // jobGenerateHelp
       } from './jobTypes';
import { initFirebase,
         initFirebaseSignIn} from './fb';

let gUI = {};
let gGLRenderer = undefined;

// TODO: delete this when the wasm work is completed
const gWasmHack = true;

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
      } else {
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
  default:
    console.log('unknown seni mode');
    break;
  }
}

function showCurrentMode(state) {
  // show the current container, hide the others
  const containers = gUI.containers;
  const currentMode = state.currentMode;

  for (let i = 0; i < SeniMode.numSeniModes; i++) {
    containers[i].className = i === currentMode ? '' : 'hidden';
  }
  showButtonsFor(currentMode);
}

function showPlaceholderImages(state) {
  const placeholder = state.placeholder;
  const populationSize = state.populationSize;
  const phenotypes = gUI.phenotypes;

  for (let i = 0; i < populationSize; i++) {
    const imageElement = phenotypes[i].imageElement;
    imageElement.src = placeholder;
  }
}

// needs the store since imageLoadHandler rebinds store.getState()
// on every image load
//
function afterLoadingPlaceholderImages(state) {
  const allImagesLoadedSince = timeStamp => {
    const phenotypes = gUI.phenotypes;

    return phenotypes.every(phenotype => {
      const imageElement = phenotype.imageElement;
      const loaded = imageElement.getAttribute('data-image-load-timestamp');
      return loaded > timeStamp;
    });
  };

  const initialTimeStamp = performance.now();

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

// update the selected phenotypes in the evolve screen according to the
// values in selectedIndices
function updateSelectionUI(state) {
  const selectedIndices = state.selectedIndices;
  const populationSize = state.populationSize;
  const phenotypes = gUI.phenotypes;

  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes[i].phenotypeElement;
    element.classList.remove('selected');
  }

  selectedIndices.forEach(i => {
    const element = phenotypes[i].phenotypeElement;
    element.classList.add('selected');
    return true;
  });
}

function renderGeometryBuffers(jobType, memory, buffers, imageElement, w, h) {
  let destWidth = undefined;
  let destHeight = undefined;
  if (w !== undefined && h !== undefined) {
    destWidth = w;
    destHeight = h;
  } else {
    destWidth = imageElement.clientWidth;
    destHeight = imageElement.clientHeight;
  }

  gGLRenderer.preDrawScene(destWidth, destHeight);

  const memoryF32 = new Float32Array(memory);

  buffers.forEach(buffer => {
    if (jobType === jobRenderWasm) {
      gGLRenderer.drawBufferFromWasm(memoryF32, buffer);
    } else {
      gGLRenderer.drawBuffer(buffer);
    }
  });

  imageElement.src = gGLRenderer.getImageData();
}

function renderGeneration(state) {
  return new Promise((resolve, _reject) => {
    const script = state.script;
    const scriptHash = state.scriptHash;

    const genotypes = state.genotypes;

    // TODO: stop generating  if the user has switched to edit mode
    const phenotypes = gUI.phenotypes;

    let hackTitle = scriptHash;

    const promises = [];

    const stopFn = startTiming();

    for (let i = 0;i < phenotypes.length; i++) {
      const workerJob = Job.request(jobRenderWasm, {
        script,
        scriptHash,
        genotype: genotypes[i]
      }).then(({ title , memory, buffers }) => {
        const imageElement = phenotypes[i].imageElement;
        renderGeometryBuffers(jobRenderWasm, memory, buffers, imageElement);
        hackTitle = title;
      }).catch(error => {
        // handle error
        console.log(`worker: error of ${error}`);
      });

      promises.push(workerJob);
    }

    Promise.all(promises).then(() => {
      stopFn(`renderGeneration-${hackTitle}`, gUI.konsole);
    }).catch(error => console.log(`renderGeneration error: ${error}`));

    resolve();
  });
}

// invoked when the evolve screen is displayed after the edit screen
function setupEvolveUI(store) {
  return new Promise((resolve, reject) => {
    afterLoadingPlaceholderImages(store.getState()).then(() => {
      return store.dispatch({type: 'INITIAL_GENERATION'});
    }).then(state => {
      // render the phenotypes
      updateSelectionUI(state);
      renderGeneration(state);
      return state;
    }).then(state => {
      return resolve(state);
    }).catch(error => {
      console.log(`setupEvolveUI error: ${error}`);
      reject(error);
    });
  });
}

function showScriptInEditor(state) {
  const editor = gUI.editor;

  editor.getDoc().setValue(state.script);
  editor.refresh();
}

function renderScript(state, imageElement) {
  const stopFn = startTiming();

  Job.request(jobRender, {
    script: state.script,
    scriptHash: state.scriptHash
  }).then(({ title, buffers }) => {
    // display any log/print messages that were generated
    // during the execution of the script
    // gUI.konsole.log(logMessages);
    renderGeometryBuffers(jobRender, null, buffers, imageElement);
    stopFn(`renderScript-${title}`, gUI.konsole);
  }).catch(error => {
    // handle error
    console.log(`worker: error of ${error}`);
  });
}

function renderScriptWithWASM(state, imageElement) {
  const stopFn = startTiming();

  Job.request(jobRenderWasm, {
    script: state.script,
    scriptHash: state.scriptHash
  }).then(({ title, memory, buffers }) => {
    // display any log/print messages that were generated
    // during the execution of the script

    // gUI.konsole.log(logMessages);
    renderGeometryBuffers(jobRenderWasm, memory, buffers, imageElement);
    stopFn(`renderScriptWithWASM-${title}`, gUI.konsole);
  }).catch(error => {
    // handle error
    console.log(`worker: error of ${error}`);
  });
}

// function that takes a read-only state and updates the UI
//
function updateUI(state) {
  showCurrentMode(state);

  switch (state.currentMode) {
  case SeniMode.gallery :
    break;
  case SeniMode.edit :
    showScriptInEditor(state);
    if (gWasmHack) {
      // WASM
      renderScriptWithWASM(state, gUI.renderImage);
    } else {
      renderScript(state, gUI.renderImage);
    }
    break;
  case SeniMode.evolve :
    // will only get here from History.restoreState
    // NOTE: the popstate event listener is handling this case
    break;
  default:
    console.log('unknown SeniMode');
    break;
  }
}

function ensureMode(store, mode) {
  return new Promise((resolve, reject) => {
    if (store.getState().currentMode === mode) {
      resolve();
      return;
    }

    store.dispatch({type: 'SET_MODE', mode}).then(state => {
      History.pushState(state);

      if (mode === SeniMode.evolve) {
        showCurrentMode(state);
        setupEvolveUI(store).then(latestState => {
          // make sure that the history for the first evolve generation
          // has the correct genotypes
          History.replaceState(latestState);
          resolve();
        }).catch(error => console.log(`ensureMode error: ${error}`));
      } else {
        updateUI(state);
        resolve();
      }
    }).catch(error => {
      console.log(`ensureMode error: ${error}`);
      reject(error);
    });
  });
}

function addClickEvent(id, fn) {
  const element = document.getElementById(id);

  if (element) {
    element.addEventListener('click', fn);
  } else {
    console.error('cannot addClickEvent for', id);
  }
}

function getIdNumberFromDom(element, regexp) {
  let e = element;
  while (e) {
    const m = e.id.match(regexp);
    if (m && m.length === 2) {
      const index = Number.parseInt(m[1], 10);
      return [index, e];
    } else {
      e = e.parentNode;
    }
  }
  return [-1, null];
}

// when user has clicked on a phenotype in the evolve UI,
// traverse up the card until we get to a dom element that
// contains the phenotype's index number in it's id
function getPhenoIdFromDom(element) {
  return getIdNumberFromDom(element, /pheno-(\d+)/);
}

function renderHighRes(state, genotype) {
  const container = document.getElementById('high-res-container');
  const loader = document.getElementById('high-res-loader');
  const image = document.getElementById('high-res-image');

  container.classList.remove('hidden');
  loader.classList.remove('hidden');
  image.classList.add('hidden');

  const stopFn = startTiming();

  Job.request(jobRenderWasm, {
    script: state.script,
    scriptHash: state.scriptHash,
    genotype: genotype ? genotype : undefined
  }).then(({ title, memory, buffers }) => {
    const [width, height] = state.highResolution;

    renderGeometryBuffers(jobRenderWasm, memory, buffers, image, width, height);

    stopFn(`renderHighRes-${title}`, gUI.konsole);

    image.classList.remove('hidden');
    const link = document.getElementById('high-res-link');
    link.href = image.src;
    loader.classList.add('hidden');
  }).catch(error => {
    // handle error
    console.log(`worker: error of ${error}`);
    gUI.konsole.log(error);
    image.classList.remove('hidden');
    loader.classList.add('hidden');
  });
}

// updates the store's script variable and then generates the traits
// in a ww and updates the store again
//
function setScript(store, script) {
  return store.dispatch({type: 'SET_SCRIPT', script});
}

function showEditFromEvolve(store, element) {
  return new Promise((resolve, reject) => {
    const [index, _] = getPhenoIdFromDom(element);
    if (index !== -1) {
      const state = store.getState();
      const genotypes = state.genotypes;

      Job.request(jobUnparse, {
        script: state.script,
        scriptHash: state.scriptHash,
        genotype: genotypes[index]
      }).then(({ script }) => {
        setScript(store, script).then(() => {
          return ensureMode(store, SeniMode.edit);
        }).then(resolve).catch(e => {
          // handle error
          console.log(`worker: error of ${e}`);
          reject(e);
        });
      }).catch(error => {
        // handle error
        console.log(`worker: error of ${error}`);
        reject(error);
      });
    } else {
      resolve();
    }
  });
}

function onNextGen(store) {
  // get the selected genotypes for the next generation
  const populationSize = store.getState().populationSize;
  const phenotypes = gUI.phenotypes;
  const selectedIndices = [];

  for (let i = 0; i < populationSize; i++) {
    const element = phenotypes[i].phenotypeElement;
    if (element.classList.contains('selected')) {
      selectedIndices.push(i);
    }
  }

  const command = {type: 'SET_SELECTED_INDICES', selectedIndices};
  store.dispatch(command).then(state => {
    if (selectedIndices.length === 0) {
      // no phenotypes were selected
      return undefined;
    }

    // update the last history state
    History.replaceState(state);

    showPlaceholderImages(state);

    return store.dispatch({type: 'NEXT_GENERATION', rng: 4242});
  }).then(state => {
    if (state === undefined) {
      return;
    }

    History.pushState(state);
    // render the genotypes
    updateSelectionUI(state);
    renderGeneration(state);
  }).catch(error => {
    // handle error
    console.log(`error of ${error}`);
  });
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

// invoked when restoring the evolve screen from the history api
function restoreEvolveUI(store) {
  return new Promise((resolve, reject) => { // todo: implement reject
    afterLoadingPlaceholderImages(store.getState()).then(() => {
      // render the phenotypes
      updateSelectionUI(store.getState());
      return renderGeneration(store.getState());
    }).then(resolve).catch(error => {
      // handle error
      console.log(`restoreEvolveUI: error of ${error}`);
      reject(error);
    });
  });
}

function showEditFromGallery(store, element) {
  return new Promise((resolve, reject) => {
    const [index, _] = getIdNumberFromDom(element, /gallery-item-(\d+)/);
    if (index !== -1) {
      const url = `/gallery/${index}`;

      get(url).catch(() => {
        reject(Error(`cannot connect to ${url}`));
      }).then(data => {
        return setScript(store, data);
      }).then(() => {
        return ensureMode(store, SeniMode.edit);
      }).then(resolve).catch(error => {
        console.log(`showEditFromGallery error ${error}`);
        reject(error);
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


function createKonsole(element) {
  const konsole = new Konsole(element, {
    prompt: '> ',
    historyLabel: 'cs-console-demo',
    syntax: 'javascript',
    initialValue: 'This is starting content\nalong with multi-lines!\n',
    welcomeMessage: 'Welcome to the cs console demo',
    autoFocus: true,
    theme: 'konsole'
  });

  // Job.request(jobGenerateHelp, {
  // }).then(documentation => {
  //   console.log('generate help returned something');
  //   const commander = new KonsoleCommander();
  //   addDefaultCommands(documentation, commander);

  //   konsole.initCallbacks({
  //     commandValidate(line) {
  //       return line.length > 0;
  //     },
  //     commandHandle(line, report, prompt) {
  //       console.log('commandHandle', line, report, prompt);
  //       commander.commandHandle(line, report, prompt);
  //     }
  //   });
  // }).catch(error => {
  //   // handle error
  //   console.log(`worker: error of ${error}`);
  // });

  return konsole;
}

function createEditor(store, editorTextArea) {
  const blockIndent = function (editor, from, to) {
    editor.operation(() => {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  const extraKeys = {
    'Ctrl-E': () => {
      setScript(store, getScriptFromEditor()).then(state => {
        if (gWasmHack) {
          return renderScriptWithWASM(state, gUI.renderImage);
        } else {
          return renderScript(state, gUI.renderImage);
        }
      }).catch(error => {
        console.log(`worker setScript error: ${error}`);
      });
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
    containers: [d.getElementById('gallery-container'),
                 d.getElementById('edit-container'),
                 d.getElementById('evolve-container')],
    // the top nav bar across the state
    navbar: d.getElementById('seni-navbar'),
    // the img destination that shows the rendered script in edit mode
    renderImage: d.getElementById('render-img'),
    // console CodeMirror element in the edit screen
    konsole: createKonsole(konsoleElement),
    editor: createEditor(store, editorTextArea)
  };

  konsoleElement.style.height = '0%';

  showButtonsFor(SeniMode.gallery);

  addClickEvent('home', event => {
    ensureMode(store, SeniMode.gallery);
    event.preventDefault();
  });

  addClickEvent('evolve-btn', event => {
    // get the latest script from the editor
    setScript(store, getScriptFromEditor()).then(state => {
      History.replaceState(state);
      ensureMode(store, SeniMode.evolve);
    }).catch(error => {
      // handle error
      console.log(`evolve-btn:click : error of ${error}`);
    });
    event.preventDefault();
  });

  addClickEvent('render-btn', event => {
    renderHighRes(store.getState());
    event.preventDefault();
  });

  addClickEvent('shuffle-btn', event => {
    showPlaceholderImages(store.getState());
    store.dispatch({type: 'SHUFFLE_GENERATION', rng: 11}).then(state => {
      updateSelectionUI(state);
      renderGeneration(state);
    }).catch(error => {
      // handle error
      console.log(`shuffle-btn:click : error of ${error}`);
    });
    event.preventDefault();
  });

  addClickEvent('eval-btn', event => {
    setScript(store, getScriptFromEditor()).then(state => {
      renderScriptWithWASM(state, gUI.renderImage);
    }).catch(error => {
      // handle error
      console.log(`eval-btn:click : error of ${error}`);
    });
    event.preventDefault();
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
        const genotypes = store.getState().genotypes;
        const genotype = genotypes[index];
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
        store.getState().currentMode === SeniMode.evolve) {
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

  const populationSize = store.getState().populationSize;
  const phenotypes = [];
  for (let i = 0; i < populationSize; i++) {
    const phenotypeElement = createPhenotypeElement(i, '');

    // get the image element
    const imageElement =
          phenotypeElement.getElementsByClassName('phenotype')[0];
    imageElement.addEventListener('load', imageLoadHandler, false);
    imageElement.setAttribute('data-image-load-timestamp', 0);

    row.appendChild(phenotypeElement);

    phenotypes.push({
      phenotypeElement,
      imageElement
    });
  }

  gUI.phenotypes = phenotypes;

  window.addEventListener('popstate', event => {
    if (event.state) {
      const savedState = History.restoreState(event.state);
      store.dispatch({type: 'SET_STATE', state: savedState}).then(state => {
        updateUI(state);
        if (state.currentMode === SeniMode.evolve) {
          restoreEvolveUI(store);
        }
      }).catch(error => {
        // handle error
        console.log(`SET_STATE: error of ${error}`);
      });
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

  document.onkeydown = evt_ => {
    const evt = evt_ || window.event;

    // Ctrl-M
    if (evt.ctrlKey && evt.keyCode === 77) {
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

function allocateWorkers(state) {
  const defaultNumWorkers = 4;
  let numWorkers = navigator.hardwareConcurrency || defaultNumWorkers;
  if (numWorkers > state.populationSize) {
    // don't allocate more workers than necessary
    numWorkers = state.populationSize;
  }
  Job.setup(numWorkers);
}

export default function main() {
  initFirebase();
  initFirebaseSignIn();

  resizeContainers();

  const state = createInitialState();
  const store = createStore(state);

  allocateWorkers(state);

  const canvasElement = document.getElementById('render-canvas');
  gGLRenderer = new GLRenderer(canvasElement);

  setupUI(store);

  getGallery().then(() => {
    return removeKonsoleInvisibility();
  }).catch(error => console.error(error));
}