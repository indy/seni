/*
 *  Seni
 *  Copyright (C) 2019 Inderjit Gill <email@indy.io>
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

// --------------------------------------------------------------------------------
// log

let logToConsole = false;

function log(msg) {
  if (logToConsole) {
    console.log(msg);
  }
}

// --------------------------------------------------------------------------------
// main

let gUI = {};
let gGLRenderer = undefined;

async function getJSON(url) {
  const res = await fetch(url);
  const json = await res.json();
  return json;
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
    log('unknown sen mode');
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

function removeAllChildren(e) {
  while (e.firstChild) {
    e.removeChild(e.firstChild);
  }
}

function removePhenotypeSpinners(state) {
  const populationSize = state.populationSize;
  const phenotypes = gUI.phenotypes;

  for (let i = 0; i < populationSize; i++) {
    const pheno = phenotypes[i];
    const phenotypeSpinner = pheno.phenotypeSpinner;
    removeAllChildren(phenotypeSpinner);
  }
}

function showPhenotypeSpinners(state) {
  const populationSize = state.populationSize;
  const phenotypes = gUI.phenotypes;

  // add the svg spinner to the first element, measure it, remove it,
  //
  const p = phenotypes[0];
  const e = p.phenotypeElement;
  // measure the initial dimensions
  let dim = e.clientWidth;
  const phenotypeSpinner = p.phenotypeSpinner;
  phenotypeSpinner.innerHTML = generateSVG(dim);
  // now get the actual dimension for all elements
  dim = e.clientWidth * 0.9;

  removeAllChildren(phenotypeSpinner);

  for (let i = 0; i < populationSize; i++) {
    const pheno = phenotypes[i];
    // remove any of the current images (e.g. if we're in the 2nd or greater generation of evolving)
    pheno.imageElement.src = "";

    const phenotypeSpinner = pheno.phenotypeSpinner;
    phenotypeSpinner.innerHTML = generateSVG(dim);
  }
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

async function renderGeometryBuffers(meta, memory, buffers, imageElement, w, h, sectionDim, section) {
  const stopFn = startTiming();

  await gGLRenderer.renderGeometryToTexture(meta, gConfig.render_texture_width, gConfig.render_texture_height, memory, buffers, sectionDim, section);
  gGLRenderer.renderTextureToScreen(meta, w, h);

  await gGLRenderer.copyImageDataTo(imageElement);

  stopFn(`rendering all buffers for section ${section}`);
}

async function renderGeneration(state) {
  // TODO: stop generating  if the user has switched to edit mode
  const script = state.script;
  const genotypes = state.genotypes;
  const phenotypes = gUI.phenotypes;
  let hackTitle = "hackTitle";
  const promises = [];

  const stopFn = startTiming();

  const dim = phenotypes[0].phenotypeElement.clientWidth;

  for (let i = 0; i < phenotypes.length; i++) {
    const workerJob = renderScript({
      script,
      genotype: genotypes[i],
      assumeWidth: dim,
      assumeHeight: dim
    }, phenotypes[i].imageElement).then(() => {
      const spinner = phenotypes[i].phenotypeSpinner;
      removeAllChildren(spinner);
    });

    promises.push(workerJob);
  }

  await Promise.all(promises);

  stopFn(`renderGeneration-${hackTitle}`);
}

// invoked when the evolve screen is displayed after the edit screen
async function setupEvolveUI(controller) {
  showPhenotypeSpinners(controller.getState());
  const state = await controller.dispatch(actionInitialGeneration);
  // render the phenotypes
  updateSelectionUI(state);
  await renderGeneration(state);

  return state;
}

function showScriptInEditor(state) {
  const editor = gUI.editor;

  editor.getDoc().setValue(state.script);
  editor.refresh();
}

// based on code from:
// https://hackernoon.com/functional-javascript-resolving-promises-sequentially-7aac18c4431e
function sequentialPromises(funcs) {
  return funcs.reduce((promise, func) =>
    promise.then(result => func().then(Array.prototype.concat.bind(result))),
    Promise.resolve([]));
}

// todo: is this the best way of getting image data for a web worker?
// is there a way for the webworker to do this without having to interact with the DOM?
// note: don't call this on a sequence of bitmaps
function loadBitmapImageData(url) {
  return new Promise((resolve, reject) => {
    const element = document.getElementById('bitmap-canvas');
    const context = element.getContext('2d');
    const img = new Image();

    img.onload = () => {
      element.width = img.width;
      element.height = img.height;

      context.drawImage(img, 0, 0);

      const imageData = context.getImageData(0, 0, element.width, element.height);

      resolve(imageData);
    };
    img.onerror = () => {
      reject();
    };

    img.src = normalize_bitmap_url(url);
  });
}

function sleepy(timeout) {
  return new Promise((resolve, reject) => {
    setTimeout(() => {
      resolve();
    }, timeout);
  });
}

function normalize_bitmap_url(url) {
  const re = /^[\w-/]+.png/;

  if (url.match(re)) {
    // requesting a bitmap just by filename, so get it from /img/immutable/
    return "img/immutable/" + url;
  } else {
    // change nothing, try and fetch the url
    return url;
  }
}

// note: this is returning a Promise
async function renderScript(parameters, imageElement) {
  const stopFn = startTiming();

  let width = parameters.assumeWidth ? parameters.assumeWidth : imageElement.clientWidth;
  let height = parameters.assumeHeight ? parameters.assumeHeight : imageElement.clientHeight;

  let { meta, memory, buffers } = await renderJob(parameters);
  await renderGeometryBuffers(meta, memory, buffers, imageElement, width, height, 1, 0);

  if (meta.title === '') {
    stopFn(`renderScript`);
  } else {
    stopFn(`renderScript-${meta.title}`);
  }
}

async function renderJob(parameters) {
  // 1. compile the program in a web worker
  // 2. (retain the id for this worker)
  // 3. after compilation, the worker will return:
  //    a: A list of textures to load onto the GPU
  //    b: A list of bitmaps that are required by the program
  //       and are not in the web worker's bitmap-cache.
  // 4. sequentially load in the bitmaps and send their data to the worker
  // 5. can now request a render which will return the render packets

  // request a compile job but make sure to retain the worker as it will be performing the rendering
  //
  parameters.__retain = true;
  const { bitmapsToTransfer, __worker_id } = await Job.request(jobRender_1_Compile, parameters);

  // convert each bitmap path to a function that returns a promise
  //
  const bitmap_loading_funcs = bitmapsToTransfer.map(filename => async () => {
    log(`worker ${__worker_id}: bitmap request: ${filename}`);

    const imageData = await loadBitmapImageData(filename);
    // make an explicit job request to the same worker
    return Job.request(jobRender_2_ReceiveBitmapData, { filename, imageData, __retain: true }, __worker_id);
  });

  // seqentially execute the promises that load in bitmaps and send the bitmap data to a particular worker
  //
  await sequentialPromises(bitmap_loading_funcs);

  // now make an explicit job request to the same worker that has recieved the bitmap data
  // note: no __retain as we want the worker to be returned to the available pool
  const renderPacketsResult = await Job.request(jobRender_3_RenderPackets, {}, __worker_id);

  return renderPacketsResult;
}

async function renderEditorScript(state) {
  const imageElement = gUI.renderImage;
  await renderScript({
    script: state.script
  }, imageElement);
}

// function that takes a read-only state and updates the UI
//
async function updateUI(state) {
  showCurrentMode(state);

  switch (state.currentMode) {
  case SeniMode.gallery :
    break;
  case SeniMode.edit :
    fitRenderImgToRenderPanel();
    showScriptInEditor(state);
    await renderEditorScript(state);
    break;
  case SeniMode.evolve :
    // will only get here from SeniHistory.restoreState
    // NOTE: the popstate event listener is handling this case
    break;
  default:
    log('unknown SeniMode');
    break;
  }
}

async function ensureMode(controller, mode) {
  if (mode === SeniMode.gallery && controller.getState().galleryLoaded === false) {
    // want to show the gallery but it hasn't been loaded yet. This occurs when
    // editing a particular sketch by loading it's id directly into the URL
    // e.g. http://localhost:3210/#61
    //
    await getGallery(controller);
  }

  if (controller.getState().currentMode !== mode) {
    const currentState = controller.getState();
    if (currentState.currentMode === SeniMode.evolve) {
      console.log('leaving evolve mode');
      removePhenotypeSpinners(currentState);
    }

    const state = await controller.dispatch(actionSetMode, { mode });
    SeniHistory.pushState(state);

    if (mode === SeniMode.evolve) {
      showCurrentMode(state);
      const latestState = await setupEvolveUI(controller);
      // make sure that the history for the first evolve generation
      // has the correct genotypes
      SeniHistory.replaceState(latestState);
    } else {
      await updateUI(state);
    }
  }
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
    if (!e.id) {
      e = e.parentNode;
    } else {
      const m = e.id.match(regexp);
      if (m && m.length === 2) {
        const index = Number.parseInt(m[1], 10);
        return [index, e];
      } else {
        e = e.parentNode;
      }
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

function downloadDialogShow() {
  const container = document.getElementById('download-dialog');
  container.classList.remove('hidden');
}

function downloadDialogHide() {
  const container = document.getElementById('download-dialog');
  container.classList.add('hidden');
}

// updates the controller's script variable and then generates the traits
// in a ww and updates the controller again
//
function setScript(controller, script) {
  return controller.dispatch(actionSetScript, { script });
}

async function showEditFromEvolve(controller, element) {
  const [index, _] = getPhenoIdFromDom(element);

  if (index !== -1) {
    const state = controller.getState();
    const genotypes = state.genotypes;
    const { script } = await Job.request(jobUnparse, {
      script: state.script,
      genotype: genotypes[index]
    });

    await controller.dispatch(actionSetScript, { script });
    await ensureMode(controller, SeniMode.edit);
  }
}

async function onNextGen(controller) {
  try {
    // get the selected genotypes for the next generation
    const populationSize = controller.getState().populationSize;
    const phenotypes = gUI.phenotypes;
    const selectedIndices = [];

    for (let i = 0; i < populationSize; i++) {
      const element = phenotypes[i].phenotypeElement;
      if (element.classList.contains('selected')) {
        selectedIndices.push(i);
      }
    }

    let state = await controller.dispatch(actionSetSelectedIndices, { selectedIndices });
    if (selectedIndices.length === 0) {
      // no phenotypes were selected
      return;
    }

    // update the last history state
    SeniHistory.replaceState(state);

    showPhenotypeSpinners(state);

    state = await controller.dispatch(actionNextGeneration, { rng: 4242 });
    if (state === undefined) {
      return;
    }

    SeniHistory.pushState(state);
    // render the genotypes
    updateSelectionUI(state);
    await renderGeneration(state);

  } catch(error) {
    // handle error
    console.error(`error of ${error}`);
  }
}

function createPhenotypeElement(id) {
  const container = document.createElement('div');

  container.className = 'card-holder';
  container.id = `pheno-${id}`;
  container.innerHTML = `
      <div id="pheno-spinner-${id}"></div>
      <a href="#">
        <img class="card-image phenotype" data-id="${id}">
      </a>
      <div class="card-action">
        <a href="#" class="render left-side">Render</a>
        <a href="#" class="edit right-side">Edit</a>
      </div>`;

  return container;
}

// invoked when restoring the evolve screen from the history api
async function restoreEvolveUI(controller) {
  showPhenotypeSpinners(controller.getState());
  updateSelectionUI(controller.getState());
  // render the phenotypes
  await renderGeneration(controller.getState());
}

async function loadScriptWithId(controller, id) {
  const response = await fetch(`gallery/${id}`);
  const script = await response.text();

  await controller.dispatch(actionSetScript, { script });
  await controller.dispatch(actionSetScriptId, { id });
  await ensureMode(controller, SeniMode.edit);
}

async function showEditFromGallery(controller, element) {
  const [index, _] = getIdNumberFromDom(element, /gallery-item-(\d+)/);
  if (index !== -1) {
    await loadScriptWithId(controller, index);
  }
}

// take the height of the navbar into consideration
function resizeContainers() {
  const navbar = document.getElementById('seni-navbar');

  const edit = document.getElementById('edit-container');
  edit.style.height = `${window.innerHeight - navbar.offsetHeight}px`;

  const evolve = document.getElementById('evolve-container');
  evolve.style.height = `${window.innerHeight - navbar.offsetHeight}px`;

  fitRenderImgToRenderPanel();
}

async function evalMainScript(controller) {
  try {
    const script = getScriptFromEditor();
    const state = await controller.dispatch(actionSetScript, { script });
    await renderEditorScript(state);
  } catch (error) {
    console.error(`evalMainScript error: ${error}`);
  }
}

function createEditor(controller, editorTextArea) {
  const blockIndent = function (editor, from, to) {
    editor.operation(() => {
      for (let i = from; i < to; ++i) {
        editor.indentLine(i, 'smart');
      }
    });
  };

  const extraKeys = {
    'Ctrl-E': async () => {
      await evalMainScript(controller);
      return false;
    },
    'Ctrl-I': () => {
      const editor = gUI.editor;
      const numLines = editor.doc.size;
      blockIndent(editor, 0, numLines);
      log(`indenting ${numLines} lines`);
      return false;
    }
  };

  return Editor.createEditor(editorTextArea, {
    theme: 'default',
    extraKeys
  });
}

function countDigits(num) {
  if(num < 10) {
    return 1;
  } else if (num < 100) {
    return 2;
  } else if (num < 1000) {
    return 3;
  } else if (num < 10000) {
    return 4;
  } else {
    console.error(`countDigits given an insanely large number: ${num}`);
    return 5;
  }
}

function filenameForPng(filename, image_dim, i) {
  // remove .png if there is any
  let name = filename.match(/\.png$/) ? filename.slice(0, -4) : filename;

  if (image_dim !== 1) {
    // add numbering to filename
    name = name + "-";

    const largestPossibleValue = (image_dim * image_dim) - 1;
    let leadingZeros = countDigits(largestPossibleValue) - countDigits(i);

    for (let i = 0; i < leadingZeros; i++) {
      name = name + "0";
    }

    name = "" + name + i;
  }

  return name + ".png";
}

function fitRenderImgToRenderPanel() {
  let smallestDim = gUI.renderPanel.clientHeight;
  if (gUI.renderPanel.clientWidth < smallestDim) {
    smallestDim = gUI.renderPanel.clientWidth;
  }

  // reduce the dimensions by 5% to provide a nicer looking gap between the renderImg and renderPanel
  smallestDim *= gUI.renderImageSizeFactor;

  gUI.renderImage.width = smallestDim;
  gUI.renderImage.height = smallestDim;
}

function setupUI(controller) {
  const d = document;
  const editorTextArea = d.getElementById('edit-textarea');

  gUI = {
    containers: [d.getElementById('gallery-container'),
                 d.getElementById('edit-container'),
                 d.getElementById('evolve-container')],
    // the top nav bar across the state
    navbar: d.getElementById('seni-navbar'),
    // the img destination that shows the rendered script in edit mode
    renderImage: d.getElementById('render-img'),
    renderPanel: d.getElementById('render-panel'),
    renderImageSizeFactor: 0.9,
    // console CodeMirror element in the edit screen
    editor: createEditor(controller, editorTextArea)
  };

  setupResizeability();

  showButtonsFor(SeniMode.gallery);

  addClickEvent('home', async event => {
    event.preventDefault();
    await ensureMode(controller, SeniMode.gallery);
  });

  addClickEvent('evolve-btn', async event => {
    try {
      event.preventDefault();
      // get the latest script from the editor
      const script = getScriptFromEditor();
      const state = await controller.dispatch(actionSetScript, { script });
      SeniHistory.replaceState(state);
      await ensureMode(controller, SeniMode.evolve);
    } catch (error) {
      // handle error
      console.error(`evolve-btn:click : error of ${error}`);
    }
  });

  addClickEvent('render-btn', event => {
    downloadDialogShow();
    event.preventDefault();
  });

  addClickEvent('shuffle-btn', async event => {
    try {
      event.preventDefault();
      showPhenotypeSpinners(controller.getState());
      const rng = Math.random() * 9999;
      const state = await controller.dispatch(actionShuffleGeneration, { rng });
      updateSelectionUI(state);
      await renderGeneration(state);
    } catch (error) {
      // handle error
      console.error(`shuffle-btn:click : error of ${error}`);
    }
  });

  addClickEvent('eval-btn', async event => {
    event.preventDefault();
    await evalMainScript(controller);
  });

  addClickEvent('gallery-list', async event => {
    event.preventDefault();
    const target = event.target;
    if (target.classList.contains('show-edit')) {
      await showEditFromGallery(controller, target);
    }
  });

  addClickEvent('evolve-container', async event => {
    const target = event.target;
    const [index, phenoElement] = getPhenoIdFromDom(target);

    event.preventDefault();
    if (target.classList.contains('render')) {
      if (index !== -1) {
        const genotypes = controller.getState().genotypes;
        const genotype = genotypes[index];

        await controller.dispatch(actionSetGenotype, { genotype });

        downloadDialogShow();
      }
    } else if (target.classList.contains('edit')) {
      showEditFromEvolve(controller, target);
    } else {
      if (index !== -1) {
        phenoElement.classList.toggle('selected');
      }
    }
  });

  addClickEvent('next-btn', () => {
    onNextGen(controller);
  });

  addClickEvent('gallery-more-btn', () => {
    createGalleryDisplayChunk(controller);
  });

  addClickEvent('download-dialog-button-ok', async event => {
    // in an async function so call preventDefault before the first await
    event.preventDefault();

    const state = controller.getState();

    const loader = document.getElementById('download-dialog-loader');
    const image = document.getElementById('render-img');

    const image_resolution_elem = document.getElementById('download-dialog-field-resolution');
    let image_resolution = parseInt(image_resolution_elem.value, 10);

    const image_dim_elem = document.getElementById('download-dialog-field-tiledim');
    let image_dim = parseInt(image_dim_elem.value, 10);

    loader.classList.remove('hidden');

    const stopFn = startTiming();

    const { meta, memory, buffers } = await renderJob({
      script: state.script,
      genotype: state.genotype,
    });

    const [width, height] = [image_resolution, image_resolution];

    for(let i = 0; i < image_dim * image_dim; i++) {
      await renderGeometryBuffers(meta, memory, buffers, image, width, height, image_dim, i);

      const image_name_elem = document.getElementById('download-dialog-field-filename');
      const filename = filenameForPng(image_name_elem.value, image_dim, i);
      gGLRenderer.localDownload(filename);
    }

    stopFn(`renderHighRes-${meta.title}`);
    loader.classList.add('hidden');

    // todo: is this the best place to reset the genotype?
    await controller.dispatch(actionSetGenotype, { genotype: undefined });
  });

  addClickEvent('download-dialog-button-close', event => {
    downloadDialogHide();
    event.preventDefault();
  });

  // Ctrl-D renders the next generation
  const dKey = 68;
  document.addEventListener('keydown', event => {
    if (event.ctrlKey && event.keyCode === dKey &&
        controller.getState().currentMode === SeniMode.evolve) {
      event.preventDefault();
      onNextGen(controller);
    }
  }, false);

  // setup the evolve-container
  const evolveGallery = document.getElementById('evolve-gallery');
  evolveGallery.innerHTML = '';

  const row = document.createElement('div');
  row.className = 'cards';
  evolveGallery.appendChild(row);

  const populationSize = controller.getState().populationSize;
  const phenotypes = [];
  for (let i = 0; i < populationSize; i++) {
    const phenotypeElement = createPhenotypeElement(i);
    // get the image element
    const imageElement = phenotypeElement.getElementsByClassName('phenotype')[0];

    row.appendChild(phenotypeElement);

    const phenotypeSpinner = document.getElementById(`pheno-spinner-${i}`);

    phenotypes.push({
      phenotypeElement,
      imageElement,
      phenotypeSpinner
    });
  }

  gUI.phenotypes = phenotypes;

  window.addEventListener('popstate', async event => {
    try {
      if (event.state) {
        const savedState = SeniHistory.restoreState(event.state);
        const state = await controller.dispatch('SET_STATE', { state: savedState });
        await updateUI(state);
        if (state.currentMode === SeniMode.evolve) {
          await restoreEvolveUI(controller);
        }
      } else {
        // no event.state so behave as if the user has visited
        // the '/' of the state
        await ensureMode(controller, SeniMode.gallery);
      }
    } catch (error) {
        // handle error
        console.error(`${actionSetState}: error of ${error}`);
    }
  });

  return controller;
}

async function getGallery(controller) {
  const galleryItems = await getJSON('gallery');

  await controller.dispatch(actionSetGalleryItems, { galleryItems });
  await createGalleryDisplayChunk(controller);
}

function generateSVG(dim) {
  const colA1 = "#000000";
  const colA2 = "#714141";
  const colB1 = "#000000";
  const colB2 = "#cf9f9f";

  // original svg downloaded from https://icons8.com/preloaders/
  return `<svg xmlns:svg="http://www.w3.org/2000/svg" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.0" width="${dim}px" height="${dim}px" viewBox="-64 -64 256 256" xml:space="preserve"><rect x="0" y="0" width="100%" height="100%" fill="#FFFFFF" /><g><linearGradient id="linear-gradient"><stop offset="0%" stop-color="${colA1}" fill-opacity="1"/><stop offset="100%" stop-color="${colA2}" fill-opacity="0.56"/></linearGradient><linearGradient id="linear-gradient2"><stop offset="0%" stop-color="${colB1}" fill-opacity="1"/><stop offset="100%" stop-color="${colB2}" fill-opacity="0.19"/></linearGradient><path d="M64 .98A63.02 63.02 0 1 1 .98 64 63.02 63.02 0 0 1 64 .98zm0 15.76A47.26 47.26 0 1 1 16.74 64 47.26 47.26 0 0 1 64 16.74z" fill-rule="evenodd" fill="url(#linear-gradient)"/><path d="M64.12 125.54A61.54 61.54 0 1 1 125.66 64a61.54 61.54 0 0 1-61.54 61.54zm0-121.1A59.57 59.57 0 1 0 123.7 64 59.57 59.57 0 0 0 64.1 4.43zM64 115.56a51.7 51.7 0 1 1 51.7-51.7 51.7 51.7 0 0 1-51.7 51.7zM64 14.4a49.48 49.48 0 1 0 49.48 49.48A49.48 49.48 0 0 0 64 14.4z" fill-rule="evenodd" fill="url(#linear-gradient2)"/><animateTransform attributeName="transform" type="rotate" from="0 64 64" to="360 64 64" dur="800ms" repeatCount="indefinite"></animateTransform></g></svg>`;
}

async function createGalleryDisplayChunk(controller) {
  const state = controller.getState();

  const createGalleryElement = item => {
    const container = document.createElement('div');
    const id = item.id;

    container.className = 'card-holder';
    container.id = `gallery-item-${id}`;

    container.innerHTML = `
      <div id="gallery-spinner-${id}"></div>
      <a href="#" class="show-edit">
        <img
             class="card-image show-edit"
             id="gallery-image-${id}"
        />
      </a>
      <div class="card-action">
        <span>${item.name}</span>
      </div>`;

    return container;
  };

  const row = document.getElementById('gallery-list-cards');
  const assumeWidth = 300;
  const assumeHeight = 300;

  let least = Math.max(state.galleryOldestToDisplay - state.galleryDisplaySize, 0);

  const promises = [];

  // append children before later code requests clientWidth
  for (let i=state.galleryOldestToDisplay; i>least; i--) {
    const item = state.galleryItems[i];
    const e = createGalleryElement(item);
    row.appendChild(e);
  }

  // add the svg spinner to the first element, measure it, remove it,
  //
  let dim = 0;
  {
    const item = state.galleryItems[state.galleryOldestToDisplay];
    const imageElement = document.getElementById(`gallery-image-${item.id}`);
    dim = imageElement.clientWidth;

    const svgContainerElement = document.getElementById(`gallery-spinner-${item.id}`);
    svgContainerElement.innerHTML = generateSVG(dim);

    dim = imageElement.clientWidth;
    removeAllChildren(svgContainerElement);
  }

  for (let i=state.galleryOldestToDisplay; i>least; i--) {
    const item = state.galleryItems[i];
    const imageElement = document.getElementById(`gallery-image-${item.id}`);
    const svgContainerElement = document.getElementById(`gallery-spinner-${item.id}`);

    svgContainerElement.innerHTML = generateSVG(dim);

    // renderScript is an async function so it will return a Promise
    //
    const workerJob = renderScript({
      script: item.script,
      assumeWidth,
      assumeHeight
    }, imageElement).then(() => {
      removeAllChildren(svgContainerElement);
    });

    promises.push(workerJob);
  }

  // console.log(`oldest id to display is now ${least}`);
  if (least === 0) {
    // hide the button
    document.getElementById('gallery-more-btn').classList.add('hidden');;
  }

  await Promise.all(promises);
  await controller.dispatch(actionGalleryOldestToDisplay, { oldestId: least});
}

function allocateWorkers(state) {
  const defaultNumWorkers = 4;
  let numWorkers = navigator.hardwareConcurrency || defaultNumWorkers;
  // let numWorkers = 1;
  if (numWorkers > state.populationSize) {
    // don't allocate more workers than necessary
    numWorkers = state.populationSize;
  }
  Job.setup(numWorkers, 'worker.js');
}

// https://developer.mozilla.org/en-US/docs/Web/Events/resize
function throttle(type, name, obj) {
  const obj2 = obj || window;
  let running = false;
  const func = () => {
    if (running) { return; }
    running = true;
    requestAnimationFrame(() => {
      obj2.dispatchEvent(new CustomEvent(name));
      running = false;
    });
  };
  obj2.addEventListener(type, func);
}

function setupResizeability() {
  // define a version of the resize event which fires less frequently
  throttle('resize', 'throttledResize');

  window.addEventListener('throttledResize', () => {
    resizeContainers();
  });

  resizeContainers();
}

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

async function loadShaders(scriptUrls) {
  const fetchPromises = scriptUrls.map(s => fetch(s));
  const responses = await Promise.all(fetchPromises);

  const textPromises = responses.map(r => r.text());
  const shaders = await Promise.all(textPromises);

  const res = {};
  for (const [i, url] of scriptUrls.entries()) {
    res[url] = shaders[i];
  }

  return res;
}

async function main() {
  const state = createInitialState();
  const controller = new Controller(state);

  allocateWorkers(state);

  const canvasElement = document.getElementById('render-canvas');

  // load the shaders asynchronously here as constructors can't do that.
  //
  const shaders = await loadShaders(['shader/main-vert.glsl',
                                     'shader/main-frag.glsl',
                                     'shader/blit-vert.glsl',
                                     'shader/blit-frag.glsl']);
  gGLRenderer = new GLRenderer(canvasElement, shaders);

  try {
    await gGLRenderer.ensureTexture(TEXTURE_UNIT_BRUSH_TEXTURE, 'brush.png');

    setupUI(controller);

    const matched = window.location.hash.match(/^\#(\d+)/);
    if (window.location.pathname === '/' && matched) {
      const id = parseInt(matched[1], 10);
      await loadScriptWithId(controller, id);
    } else {
      await ensureMode(controller, SeniMode.gallery);
    }
  } catch (error) {
    console.error(error);
  }
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
