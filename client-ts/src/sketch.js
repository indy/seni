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

const RUNNING_ON_STATIC_SITE = false;
const PAGE = RUNNING_ON_STATIC_SITE ? "index.html" : "sketch.html";
const PREFIX = RUNNING_ON_STATIC_SITE ? "/app/www" : "";

// --------------------------------------------------------------------------------

const URI_SEED = "seed";
const URI_MODE = "mode";

const MODE_NORMAL = "normal";
const MODE_SLIDESHOW = "slideshow";

// either display the generated image asap or fade it in
const DISPLAY_SNAP = 0;
const DISPLAY_FADE = 1;

const IMG_0 = 'sketch-img-0';
const IMG_1 = 'sketch-img-1';

let gState = {
  glRenderer: undefined,
  logDebug: false,
  timoutId: undefined,

  slideshowDelay: 5000,
  demandCanvasSize: 500,
  mode: MODE_NORMAL,
  seed: undefined,
  activeImageElement: 0,
  lastDisplay: DISPLAY_SNAP,

  render_texture_width: 1024,
  render_texture_height: 1024,
};

function logDebug(msg) {
  if (gState.logDebug) {
    const op0 = getRequiredElement(IMG_0).style.opacity;
    const op1 = getRequiredElement(IMG_1).style.opacity;

    console.log(`${msg} ${gState.mode} img-0 opacity: ${op0}, img-1 opacity: ${op1} activeImageElement: ${gState.activeImageElement}`);
  }
}

async function displayOnImageElements(display) {
  // required to check that an endAnimation doesn't fade in sketch-img-1
  gState.lastDisplay = display;

  if (display === DISPLAY_SNAP) {
    resetImageElements();

    const sketchImg0 = getRequiredElement(IMG_0);
    await gState.glRenderer.copyImageDataTo(sketchImg0);
  } else {
    if (gState.activeImageElement === 0) {
      const sketchImg0 = getRequiredElement(IMG_0);
      await gState.glRenderer.copyImageDataTo(sketchImg0);

      if (gState.mode === MODE_NORMAL) {
        addClass(IMG_1, 'seni-fade-out');
      } else if (gState.mode === MODE_SLIDESHOW) {
        addClass(IMG_1, 'seni-fade-out-slideshow');
      }
    } else {
      const sketchImg1 = getRequiredElement(IMG_1);
      await gState.glRenderer.copyImageDataTo(sketchImg1);

      if (gState.mode === MODE_NORMAL) {
        addClass(IMG_1, 'seni-fade-in');
      } else if (gState.mode === MODE_SLIDESHOW) {
        addClass(IMG_1, 'seni-fade-in-slideshow');
      }
    }

    gState.activeImageElement = 1 - gState.activeImageElement;
  }

  logDebug("displayOnImageElements");
}

async function renderGeometryBuffers(meta, memory, buffers, destWidth, destHeight, display) {
  await gState.glRenderer.renderGeometryToTexture(meta, gState.render_texture_width, gState.render_texture_height, memory, buffers, 1, 0);
  gState.glRenderer.renderTextureToScreen(meta, destWidth, destHeight);

  await displayOnImageElements(display);
}


async function renderScript(parameters, display) {
  console.log(`renderScript  (demandCanvasSize = ${gState.demandCanvasSize})`);
  let { meta, memory, buffers } = await renderJob(parameters);
  await renderGeometryBuffers(meta, memory, buffers, gState.demandCanvasSize, gState.demandCanvasSize, display);
}

function getSeedValue(element) {
  const res = parseInt(element.value, 10);
  return res;
}

async function showSimplifiedScript(fullScript) {
  const { script } = await Job.request(JobType.jobSimplifyScript, {
    script: fullScript
  });

  const simplifiedScriptElement = getRequiredElement('sketch-simplified-script');
  simplifiedScriptElement.textContent = script;
}

function showId(id) {
  removeClass(id, 'seni-hide');
}

function hideId(id) {
  addClass(id, 'seni-hide');
}

async function performSlideshow() {
  if (gState.mode === MODE_SLIDESHOW) {
    const scriptElement = getRequiredElement('sketch-script');
    const seedElement = getRequiredElement('sketch-seed');
    const script = scriptElement.textContent;
    const originalScript = script.slice();

    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);
    gState.seed = getSeedValue(seedElement);

    updateURIFromGlobals(false);
    await updateSketch(DISPLAY_FADE);
    gState.timeoutId = window.setTimeout(performSlideshow, gState.slideshowDelay);
  }
}

function getCSSAnimationDuration(className) {
  const indyioCSSStylesheet = 0; // note: update this if more than one stylesheet is used

  const styleSheet = document.styleSheets[indyioCSSStylesheet];

  let cssRules = undefined;
  for(let i = 0; i < styleSheet.cssRules.length; i++) {
    if (styleSheet.cssRules[i].selectorText === className) {
      cssRules = styleSheet.cssRules[i];
      return parseFloat(cssRules.style.animationDuration);
    }
  }
  return undefined;
}

function resetImageElements() {
  setOpacity(IMG_1, 0);
  gState.activeImageElement = 1;

  removeClass(IMG_1, 'seni-fade-in');
  removeClass(IMG_1, 'seni-fade-in-slideshow');
}

function moveContainerInsideParent(parentId, forceLargest) {
  const canvasContainerId = 'sketch-canvas-container';
  const canvasContainer = getRequiredElement(canvasContainerId);

  const parent = getRequiredElement(parentId);
  parent.appendChild(canvasContainer);

  let dim = 0;
  if (forceLargest) {
    let forceWidth = document.documentElement.clientWidth;
    let forceHeight = document.documentElement.clientHeight;
    dim = forceWidth < forceHeight ? forceWidth : forceHeight;


    let marginLeft = (forceWidth - dim) / 2;
    canvasContainer.style.marginLeft = "" + marginLeft + "px";

  } else {
    dim = parent.clientWidth < parent.clientHeight ? parent.clientWidth : parent.clientHeight;
    canvasContainer.style.marginLeft = "0px";
  }

  canvasContainer.width = dim;
  canvasContainer.height = dim;
  gState.demandCanvasSize = dim;

  const img0 = getRequiredElement('sketch-img-0');
  img0.width = dim;
  img0.height = dim;

  const img1 = getRequiredElement('sketch-img-1');
  img1.width = dim;
  img1.height = dim;
}

function styleForNormalSketch() {
  showId('header');
  showId('main');

  moveContainerInsideParent('sketch-normal-anchor');

  resetImageElements();
}

function styleForLargeSketch() {
  hideId('header');
  hideId('main');

  moveContainerInsideParent('sketch-large-anchor', true);

  resetImageElements();
}


async function updateToMode(newMode) {
  if (gState.mode === newMode) {
    return false;
  }

  gState.mode = newMode;

  gState.glRenderer.clear();

  const sketchImg0 = getRequiredElement(IMG_0);
  await gState.glRenderer.copyImageDataTo(sketchImg0);
  const sketchImg1 = getRequiredElement(IMG_1);
  await gState.glRenderer.copyImageDataTo(sketchImg1);

  if (gState.mode === MODE_SLIDESHOW) {
    styleForLargeSketch();
  } else if (gState.mode === MODE_NORMAL) {
    window.clearTimeout(gState.timeoutId); // stop the slideshow
    styleForNormalSketch();
  }

  return true;
}

function animationEndListener1(event) {
  if (event.animationName === 'senifadeout') {
    removeClass(IMG_1, 'seni-fade-out');
    removeClass(IMG_1, 'seni-fade-out-slideshow');
    setOpacity(IMG_1, 0);
  }

  if (event.animationName === 'senifadein') {
    removeClass(IMG_1, 'seni-fade-in');
    removeClass(IMG_1, 'seni-fade-in-slideshow');
    if (gState.lastDisplay === DISPLAY_SNAP) {
      // if we were in a slideshow and the user pressed escape to go back to a normal render
      // the fade animation that was playing for the previous mode has now finished
      setOpacity(IMG_1, 0);
    } else {
      setOpacity(IMG_1, 1);
    }
  }
}

function updateGlobalsFromURI() {
  const uriParameters = getURIParameters();

  if (uriParameters.hasOwnProperty(URI_SEED)) {
    gState.seed = uriParameters[URI_SEED];
  } else {
    gState.seed = undefined;
  }

  if (uriParameters[URI_MODE] === MODE_SLIDESHOW) {
    updateToMode(MODE_SLIDESHOW);
  } else {
    // absence of mode parameter in URI means MODE_NORMAL
    updateToMode(MODE_NORMAL);
  }
}

function updateURIFromGlobals(updateHistory) {
  let params = [];
  if (gState.mode != MODE_NORMAL) {
    params.push("mode=" + gState.mode);
  }
  if (gState.seed !== undefined) {
    params.push("seed=" + gState.seed);
  }

  let search = "";
  if (params.length > 0) {
    search = "?" + params.join("&");
  }

  if (updateHistory && window.location.search !== search) {
    // desired uri is different from current one
    const page_uri = PAGE + search;
    history.pushState({}, null, page_uri);
  }
}

async function renderNormal(display) {
  const scriptElement = getRequiredElement('sketch-script');
  const script = scriptElement.textContent.slice();

  if (gState.seed === undefined) {
    await showSimplifiedScript(script);
    await renderScript({ script }, display);
  } else {
    const { traits } = await Job.request(JobType.jobBuildTraits, { script });
    const { genotype } = await Job.request(JobType.jobSingleGenotypeFromSeed, { traits, seed: gState.seed });

    const unparsed = await Job.request(JobType.jobUnparse, { script, genotype });
    await showSimplifiedScript(unparsed.script);
    await renderScript({ script, genotype }, display);
  }
}

async function updateSketch(display) {
  await renderNormal(display);
}

async function main() {
  updateGlobalsFromURI();

  Job.setup(2, `${PREFIX}/worker.js`);

  const originalButton = getRequiredElement('sketch-eval-original');
  const variationButton = getRequiredElement('sketch-eval-variation');
  const slideshowButton = getRequiredElement('sketch-eval-slideshow');
  const scriptElement = getRequiredElement('sketch-script');
  const canvasElement = getRequiredElement('sketch-canvas');
  const canvasImageElement0 = getRequiredElement(IMG_0);
  const canvasImageElement1 = getRequiredElement(IMG_1);

  canvasImageElement1.addEventListener("animationend", animationEndListener1, false);
  setOpacity(IMG_1, 0);

  const shaders = await loadShaders(['shader/main-vert.glsl',
                                     'shader/main-frag.glsl',
                                     'shader/blit-vert.glsl',
                                     'shader/blit-frag.glsl']);
  gState.glRenderer = new GLRenderer(canvasElement, shaders);

  const script = scriptElement.textContent;
  const originalScript = script.slice();

  logDebug("init");

  await gState.glRenderer.ensureTexture(TEXTURE_UNIT_BRUSH_TEXTURE, `brush.png`);
  await updateSketch(DISPLAY_SNAP);

  // gState.glRenderer.loadTexture(`${PREFIX}/img/brush.png`)
  //   .then(async () => await updateSketch(DISPLAY_SNAP))
  //   .catch(error => console.error(error));

  originalButton.addEventListener('click', async () => {
    originalButton.disabled = true;

    gState.seed = undefined;
    updateToMode(MODE_NORMAL);

    updateURIFromGlobals(true);

    await updateSketch(DISPLAY_FADE);
  });

  slideshowButton.addEventListener('click', async () => {
    originalButton.disabled = false;

    if (updateToMode(MODE_SLIDESHOW)) {
      await updateSketch(DISPLAY_SNAP);
      const sketchImg1 = getRequiredElement(IMG_1);
      await gState.glRenderer.copyImageDataTo(sketchImg1);

      // only call updateSketch if we're actually switching to SLIDESHOW mode as this will create a settimeout
      gState.timeoutId = window.setTimeout(performSlideshow, 0);
    }
    updateURIFromGlobals(true);

  });

  variationButton.addEventListener('click', async () => {
    originalButton.disabled = false;

    const seedElement = getRequiredElement('sketch-seed');
    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);
    gState.seed = getSeedValue(seedElement);

    updateToMode(MODE_NORMAL);

    updateURIFromGlobals(true);

    await updateSketch(DISPLAY_FADE);
  });

  window.addEventListener('popstate', async event => {
    updateGlobalsFromURI();
    await updateSketch(DISPLAY_SNAP);
  });

  canvasImageElement1.addEventListener('click', async () => {
    updateToMode(MODE_NORMAL);

    updateURIFromGlobals(true);

    await updateSketch(DISPLAY_SNAP);
  });

  const escapeKey = 27;
  document.addEventListener('keydown', async event => {
    if (event.keyCode === escapeKey && gState.mode !== MODE_NORMAL) {

      updateToMode(MODE_NORMAL);

      updateURIFromGlobals(true);

      await updateSketch(DISPLAY_SNAP);

      event.preventDefault();
    }
  }, false);
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
