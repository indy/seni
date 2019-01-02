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

import GLRenderer from './seni/GLRenderer';
import Util from './seni/Util';
import Job from './job';
import { jobRender,
         jobUnparse,
         jobBuildTraits,
         jobSingleGenotypeFromSeed,
         jobSimplifyScript
       } from './jobTypes';
let gGLRenderer = undefined;


let gLogDebug = false;
let gTimeoutId = undefined;
let gSlideshowDelay = 5000;
let gDemandCanvasSize = 500;
let gMode = "normal";           // normal | slideshow
let gActiveImageElement = 0;
let gNumTransitions = 0;        // reset after every mode switch


function logDebug(msg) {
  if (gLogDebug) {
    const op0 = getRequiredElement('piece-img-0').style.opacity;
    const op1 = getRequiredElement('piece-img-1').style.opacity;

    console.log(`${msg} ${gMode} gNumTransitions: ${gNumTransitions} img-0 opacity: ${op0}, img-1 opacity: ${op1} activeImageElement: ${gActiveImageElement}`);
  }
}

function updatePieceDimensions(pieceImg, canvas, w, h) {
  pieceImg.style.top = canvas.offsetTop + "px";
  pieceImg.style.left = canvas.offsetLeft + "px";
  pieceImg.width = w;
  pieceImg.height = h;
}

function updatePieceData(pieceImg) {
  pieceImg.src = gGLRenderer.getImageData();
}

function displayOnImageElements() {
  const canvas = getRequiredElement('piece-canvas');
  const pieceImg0 = getRequiredElement('piece-img-0');
  const pieceImg1 = getRequiredElement('piece-img-1');

  if (gNumTransitions === 0) {
    // have just switched modes, so make sure the images are correctly positioned
    setOpacity('piece-img-0', 1);
    updatePieceDimensions(pieceImg0, canvas, gDemandCanvasSize, gDemandCanvasSize);
    updatePieceDimensions(pieceImg1, canvas, gDemandCanvasSize, gDemandCanvasSize);
  }

  if (gActiveImageElement === 0) {
    updatePieceData(pieceImg0);
    if (gNumTransitions > 0) {
      if (gMode === "normal") {
        addClass('piece-img-1', 'seni-fade-out');
      } else {
        addClass('piece-img-1', 'seni-fade-out-slideshow');
      }
    }

  } else {
    updatePieceData(pieceImg1);
    if (gNumTransitions > 0) {
      if (gMode === "normal") {
        addClass('piece-img-1', 'seni-fade-in');
      } else {
        addClass('piece-img-1', 'seni-fade-in-slideshow');
      }
    }
  }

  gActiveImageElement = 1 - gActiveImageElement;

  logDebug("displayOnImageElements");
}

function renderBuffers(memory, buffers, w, h) {
  // this will update the size of the piece-canvas element
  gGLRenderer.preDrawScene(w, h);

  const memoryF32 = new Float32Array(memory);

  buffers.forEach(buffer => {
    gGLRenderer.drawBuffer(memoryF32, buffer);
  });

  displayOnImageElements();
}

function renderScript(config) {
  return Job.request(jobRender, config)
    .then(({ memory, buffers }) => {
      renderBuffers(memory, buffers, gDemandCanvasSize, gDemandCanvasSize);
    }).catch(error => {
      // handle error
      console.error(`worker: error of ${error}`);
    });
}

function buildTraits(config) {
  return Job.request(jobBuildTraits, config);
}

function buildGenotype(config) {
  return Job.request(jobSingleGenotypeFromSeed, config);
}

function unparse(config) {
  return Job.request(jobUnparse, config);
}

function getSeedValue(element) {
  const res = parseInt(element.value, 10);
  return res;
}

function fetchScript(id) {
  return fetch(`/gallery/${id}`).then(response => response.text());
}

function getRequiredElement(id) {
  const element = document.getElementById(id);
  if (!element) {
    console.error(`required element ${id} not found in dom`);
  }
  return element;
}

function showSimplifiedScript(fullScript) {
  Job.request(jobSimplifyScript, {
    script: fullScript
  }).then(({ script }) => {
    const simplifiedScriptElement =
          getRequiredElement('piece-simplified-script');
    simplifiedScriptElement.textContent = script;
  }).catch(error => {
    // handle error
    console.error(`worker: error of ${error}`);
  });
}

function useLargeCanvas() {
  gDemandCanvasSize = window.innerWidth < window.innerHeight ? window.innerWidth : window.innerHeight;
  gDemandCanvasSize *= 0.9;
}

function useNormalCanvas() {
  gDemandCanvasSize = 500;
}

function addClass(id, clss) {
  const e = getRequiredElement(id);
  e.classList.add(clss);
}

function removeClass(id, clss) {
  const e = getRequiredElement(id);
  e.classList.remove(clss);
}

function showId(id) {
  removeClass(id, 'seni-hide');
}

function hideId(id) {
  addClass(id, 'seni-hide');
}

function setOpacity(id, opacity) {
  const e = getRequiredElement(id);
  e.style.opacity = opacity;
}

function performSlideshow() {
  gNumTransitions += 1;
  const scriptElement = getRequiredElement('piece-script');
  const seedElement = getRequiredElement('piece-seed');

  const scriptHash = Util.hashCode('whatever');

  const script = scriptElement.textContent;
  const originalScript = script.slice();

  const newSeed = Math.random() * (1 << 30);
  seedElement.value = parseInt(newSeed, 10);

  const seedValue = getSeedValue(seedElement);
  buildTraits({ script: originalScript, scriptHash })
    .then(({ traits }) => buildGenotype({ traits, seed: seedValue }))
    .then(({ genotype }) => {
      const config = { script: originalScript, scriptHash };
      if (seedValue !== 0) {
        config.genotype = genotype;
      }
      return renderScript(config);
    })
    .then(() => {
      gTimeoutId = window.setTimeout(performSlideshow, gSlideshowDelay);
    })
    .catch(error => {
      console.error('performSlideshow error');
      console.error(error);
    });
}

// returns true if the mode was actually changed
//
function setMode(newMode) {
  if (newMode === "normal" && gMode !== "normal") {
    gMode = "normal";
    window.clearTimeout(gTimeoutId); // stop the slideshow
    addClass('piece-content', 'piece-content-wrap');
    useNormalCanvas();
    showId('header');
    showId('seni-piece-controls');
    showId('code-content-wrap');
    showId('seni-title');
    showId('seni-date');
    showId('piece-hideable-for-slideshow');
    removeClass('piece-canvas-container', 'seni-centre-canvas');

    setOpacity('piece-img-0', 0);
    setOpacity('piece-img-1', 0);
    gActiveImageElement = 0;
    gNumTransitions = 0;


    const originalButton = getRequiredElement('piece-eval-original');
    const scriptElement = getRequiredElement('piece-script');

    const scriptHash = Util.hashCode('whatever');
    const script = scriptElement.textContent;
    const originalScript = script.slice();

    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    showSimplifiedScript(originalScript);
    originalButton.disabled = true;

    return true;
  } else if (newMode === "slideshow" && gMode !== "slideshow") {
    gMode = "slideshow";

    removeClass('piece-content', 'piece-content-wrap');
    useLargeCanvas();
    hideId('header');
    hideId('seni-piece-controls');
    hideId('code-content-wrap');
    hideId('seni-title');
    hideId('seni-date');
    hideId('piece-hideable-for-slideshow');
    addClass('piece-canvas-container', 'seni-centre-canvas');

    setOpacity('piece-img-0', 0);
    setOpacity('piece-img-1', 0);
    gActiveImageElement = 0;
    gNumTransitions = 0;

    gTimeoutId = window.setTimeout(performSlideshow, 1500);
    return true;
  }
  return false;
}

function animationEndListener(event, id) {
  if (event.animationName === 'senifadeout') {
    removeClass(id, 'seni-fade-out');
    removeClass(id, 'seni-fade-out-slideshow');
    setOpacity(id, 0);
  }

  if (event.animationName === 'senifadein') {
    removeClass(id, 'seni-fade-in');
    removeClass(id, 'seni-fade-in-slideshow');
    setOpacity(id, 1);
  }
}

function animationEndListener1(event) {
  animationEndListener(event, 'piece-img-1');
}

export default function main() {

  const texturePathElement = getRequiredElement('piece-texture-path');
  const workerPathElement = getRequiredElement('piece-worker-path');

  Job.setup(2, workerPathElement.textContent);

  const originalButton = getRequiredElement('piece-eval-original');
  const evalButton = getRequiredElement('piece-eval');
  const slideshowButton = getRequiredElement('piece-eval-slideshow');
  const scriptElement = getRequiredElement('piece-script');
  const canvasElement = getRequiredElement('piece-canvas');
  const canvasImageElement0 = getRequiredElement('piece-img-0');
  const canvasImageElement1 = getRequiredElement('piece-img-1');
  const seedElement = getRequiredElement('piece-seed');

  canvasImageElement1.addEventListener("animationend", animationEndListener1, false);
  setOpacity('piece-img-0', 0);
  setOpacity('piece-img-1', 0);

  if (LOAD_FOR_SENI_APP_GALLERY === false) {
    // not really required, hack to load in other pieces
    const loadIdElement = getRequiredElement('piece-load-id');
    loadIdElement.addEventListener('change', event => {
      console.log('loadidelement');
      const iVal = parseInt(event.target.value, 10);

      fetchScript(iVal).then(code => {
        script = code;
        originalScript = script.slice();
        scriptElement.textContent = script;
        showSimplifiedScript(script);
        return renderScript({ script, scriptHash });
      }).catch(error => console.error(error));
    });
  }

  gMode = "normal";

  const scriptHash = Util.hashCode('whatever');

  gGLRenderer = new GLRenderer(canvasElement);

  const script = scriptElement.textContent;
  const originalScript = script.slice();
  showSimplifiedScript(script);

  logDebug("init");

  gGLRenderer.loadTexture(texturePathElement.textContent)
    .then(() => renderScript({ script, scriptHash }))
    .catch(error => console.error(error));

  originalButton.addEventListener('click', () => {
    setMode("normal");
    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    showSimplifiedScript(originalScript);
    originalButton.disabled = true;
  });

  slideshowButton.addEventListener('click', () => {
    setMode("slideshow");
    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    showSimplifiedScript(originalScript);
    originalButton.disabled = true;
  });

  evalButton.addEventListener('click', () => {
    gNumTransitions += 1;
    originalButton.disabled = false;
    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);

    const seedValue = getSeedValue(seedElement);
    buildTraits({ script: originalScript, scriptHash })
      .then(({ traits }) => buildGenotype({ traits, seed: seedValue }))
      .then(({ genotype }) => {
        const config = { script: originalScript, scriptHash };
        if (seedValue !== 0) {
          config.genotype = genotype;
        }
        renderScript(config);

        return unparse({ script: originalScript, genotype });
      })
      .then(({ script }) => {
        scriptElement.textContent = script;
        showSimplifiedScript(script);
      })
      .catch(error => {
        console.error('piece-eval click error');
        console.error(error);
      });
  });

  canvasImageElement1.addEventListener('click', () => {
    setMode("normal");
  });

  const escapeKey = 27;
  document.addEventListener('keydown', event => {
    if (event.keyCode === escapeKey && gMode === 'slideshow') {
      setMode('normal');
      event.preventDefault();
    }
  }, false);
}
