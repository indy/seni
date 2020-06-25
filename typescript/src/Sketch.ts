/*
 *  Copyright (C) 2020 Inderjit Gill <email@indy.io>
 *
 *  This file is part of Seni
 *
 *  Seni is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Seni is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

///<reference path='SeniMode.ts'/>
///<reference path='State.ts'/>
///<reference path='Log.ts'/>
///<reference path='Renderer.ts'/>
///<reference path='Timer.ts'/>
///<reference path='Job.ts'/>
///<reference path='utils.js'/>

const RUNNING_ON_STATIC_SITE = true;
const PAGE = RUNNING_ON_STATIC_SITE ? "index.html" : "sketch.html";
const PREFIX = RUNNING_ON_STATIC_SITE ? "/app/www" : "";

// --------------------------------------------------------------------------------

const URI_SEED = "seed";
const URI_MODE = "mode";

enum SketchMode {
    normal = 0,
    slideshow,
}

enum DisplayMode {
    snap = 0,
    fade,
}

const IMG_0 = 'sketch-img-0';
const IMG_1 = 'sketch-img-1';

let g_render_texture_width: number = 2048;
let g_render_texture_height: number = 2048;
let gGLRenderer: GLRenderer2;
let gTimeoutId: number;
let gSeed: number;
let gMode: SketchMode;
let gLastDisplay: DisplayMode;

let gState = {
    logDebug: false,

    slideshowDelay: 5000,
    demandCanvasSize: 500,
    activeImageElement: 0,
    // render_texture_width: 1024,
    // render_texture_height: 1024,
};

function logDebug(msg: string) {
    if (gState.logDebug) {
        const op0 = getRequiredElement(IMG_0)!.style.opacity;
        const op1 = getRequiredElement(IMG_1)!.style.opacity;

        console.log(`${msg} ${gMode} img-0 opacity: ${op0}, img-1 opacity: ${op1} activeImageElement: ${gState.activeImageElement}`);
    }
}

async function displayOnImageElements(display: DisplayMode) {
    // required to check that an endAnimation doesn't fade in sketch-img-1
    gLastDisplay = display;

    if (display === DisplayMode.snap) {
        resetImageElements();

        const sketchImg0 = <HTMLImageElement>getRequiredElement(IMG_0);
        await gGLRenderer.copyImageDataTo(sketchImg0);
    } else {
        if (gState.activeImageElement === 0) {
            const sketchImg0 = <HTMLImageElement>getRequiredElement(IMG_0);
            await gGLRenderer.copyImageDataTo(sketchImg0);

            if (gMode === SketchMode.normal) {
                addClass(IMG_1, 'seni-fade-out');
            } else if (gMode === SketchMode.slideshow) {
                addClass(IMG_1, 'seni-fade-out-slideshow');
            }
        } else {
            const sketchImg1 = <HTMLImageElement>getRequiredElement(IMG_1);
            await gGLRenderer.copyImageDataTo(sketchImg1);

            if (gMode === SketchMode.normal) {
                addClass(IMG_1, 'seni-fade-in');
            } else if (gMode === SketchMode.slideshow) {
                addClass(IMG_1, 'seni-fade-in-slideshow');
            }
        }

        gState.activeImageElement = 1 - gState.activeImageElement;
    }

    logDebug("displayOnImageElements");
}

async function renderGeometryBuffers(meta: any, memory: any, buffers: any, destWidth: number, destHeight: number, display: DisplayMode) {
    await gGLRenderer.renderGeometryToTexture(PREFIX, meta, g_render_texture_width, g_render_texture_height, memory, buffers, 1, 0);
    gGLRenderer.renderTextureToScreen(meta, destWidth, destHeight);

    await displayOnImageElements(display);
}


async function renderScript(parameters: any, display: DisplayMode) {
    console.log(`renderScript  (demandCanvasSize = ${gState.demandCanvasSize})`);
    let { meta, memory, buffers } = await renderJob(parameters);
    await renderGeometryBuffers(meta, memory, buffers, gState.demandCanvasSize, gState.demandCanvasSize, display);
}

function getSeedValue(element: HTMLInputElement) {
    const res = parseInt(element.value, 10);
    return res;
}

async function showSimplifiedScript(fullScript: string) {
    const { script } = await Job.request(JobType.jobSimplifyScript, {
        script: fullScript
    }, undefined);

    const simplifiedScriptElement = <HTMLElement>getRequiredElement('sketch-simplified-script');
    simplifiedScriptElement.textContent = script;
}

function showId(id: any) {
    removeClass(id, 'seni-hide');
}

function hideId(id: any) {
    addClass(id, 'seni-hide');
}

async function performSlideshow() {
    if (gMode === SketchMode.slideshow) {
        // const scriptElement = <HTMLElement>getRequiredElement('sketch-script');
        const seedElement = <HTMLInputElement>getRequiredElement('sketch-seed');
        // const script = scriptElement.textContent;
        // const originalScript = script.slice();

        const newSeed = Math.random() * (1 << 30);
        seedElement.value = newSeed.toString();
        gSeed = getSeedValue(seedElement);

        updateURIFromGlobals(false);
        await updateSketch(DisplayMode.fade);
        gTimeoutId = window.setTimeout(performSlideshow, gState.slideshowDelay);
    }
}

// function getCSSAnimationDuration(className) {
//     const indyioCSSStylesheet = 0; // note: update this if more than one stylesheet is used

//     const styleSheet = document.styleSheets[indyioCSSStylesheet];

//     let cssRules = undefined;
//     for(let i = 0; i < styleSheet.cssRules.length; i++) {
//         if (styleSheet.cssRules[i].selectorText === className) {
//             cssRules = styleSheet.cssRules[i];
//             return parseFloat(cssRules.style.animationDuration);
//         }
//     }
//     return undefined;
// }

function resetImageElements() {
    setOpacity(IMG_1, 0);
    gState.activeImageElement = 1;

    removeClass(IMG_1, 'seni-fade-in');
    removeClass(IMG_1, 'seni-fade-in-slideshow');
}

function moveContainerInsideParent(parentId: string, forceLargest: boolean) {
    const canvasContainerId = 'sketch-canvas-container';
    const canvasContainer = <HTMLElement>getRequiredElement(canvasContainerId);

    const parent = <HTMLElement>getRequiredElement(parentId);
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

    // canvasContainer.width = dim;  // note: 25/06/2020 uncomment?
    // canvasContainer.height = dim;
    gState.demandCanvasSize = dim;

    const img0 = <HTMLImageElement>getRequiredElement('sketch-img-0');
    img0.width = dim;
    img0.height = dim;

    const img1 = <HTMLImageElement>getRequiredElement('sketch-img-1');
    img1.width = dim;
    img1.height = dim;
}

function styleForNormalSketch() {
    showId('header');
    showId('main');

    moveContainerInsideParent('sketch-normal-anchor', false);

    resetImageElements();
}

function styleForLargeSketch() {
    hideId('header');
    hideId('main');

    moveContainerInsideParent('sketch-large-anchor', true);

    resetImageElements();
}


async function updateToMode(newMode: SketchMode) {
    if (gMode === newMode) {
        return false;
    }

    gMode = newMode;

    gGLRenderer.clear();

    const sketchImg0 = <HTMLImageElement>getRequiredElement(IMG_0);
    await gGLRenderer.copyImageDataTo(sketchImg0);
    const sketchImg1 = <HTMLImageElement>getRequiredElement(IMG_1);
    await gGLRenderer.copyImageDataTo(sketchImg1);

    if (gMode === SketchMode.slideshow) {
        styleForLargeSketch();
    } else if (gMode === SketchMode.normal) {
        window.clearTimeout(gTimeoutId); // stop the slideshow
        styleForNormalSketch();
    }

    return true;
}

function animationEndListener1(event: any) {
    if (event.animationName === 'senifadeout') {
        removeClass(IMG_1, 'seni-fade-out');
        removeClass(IMG_1, 'seni-fade-out-slideshow');
        setOpacity(IMG_1, 0);
    }

    if (event.animationName === 'senifadein') {
        removeClass(IMG_1, 'seni-fade-in');
        removeClass(IMG_1, 'seni-fade-in-slideshow');
        if (gLastDisplay === DisplayMode.snap) {
            // if we were in a slideshow and the user pressed escape to go back to a normal render
            // the fade animation that was playing for the previous mode has now finished
            setOpacity(IMG_1, 0);
        } else {
            setOpacity(IMG_1, 1);
        }
    }
}

function updateGlobalsFromURI() {
    const uriParameters: {[index: string]:any} = getURIParameters();

    if (uriParameters.hasOwnProperty(URI_SEED)) {
        gSeed = <number>uriParameters[URI_SEED];
    } else {
        gSeed = 0;
    }

    if (uriParameters[URI_MODE] === SketchMode.slideshow) {
        updateToMode(SketchMode.slideshow);
    } else {
        // absence of mode parameter in URI means SketchMode.normal
        updateToMode(SketchMode.normal);
    }
}

function updateURIFromGlobals(updateHistory: boolean) {
    let params = [];
    if (gMode != SketchMode.normal) {
        params.push("mode=" + gMode);
    }
    if (gSeed !== undefined) {
        params.push("seed=" + gSeed);
    }

    let search = "";
    if (params.length > 0) {
        search = "?" + params.join("&");
    }

    if (updateHistory && window.location.search !== search) {
        // desired uri is different from current one
        const page_uri = PAGE + search;
        history.pushState({}, "", page_uri);
    }
}

async function renderNormal(display: DisplayMode) {
    const scriptElement = <HTMLElement>getRequiredElement('sketch-script');
    const script = scriptElement.textContent!.slice();

    if (gSeed === undefined) {
        await showSimplifiedScript(script);
        await renderScript({ script }, display);
    } else {
        const { traits } = await Job.request(JobType.jobBuildTraits, { script }, undefined);
        const { genotype } = await Job.request(JobType.jobSingleGenotypeFromSeed, { traits, seed: gSeed }, undefined);

        const unparsed = await Job.request(JobType.jobUnparse, { script, genotype }, undefined);
        await showSimplifiedScript(unparsed.script);
        await renderScript({ script, genotype }, display);
    }
}

async function updateSketch(display: DisplayMode) {
    await renderNormal(display);
}

async function main() {
    gMuhPrefix = PREFIX;

    gLastDisplay = DisplayMode.snap;

    Job.setup(2, `${PREFIX}/worker.js`);

    const originalButton = <HTMLButtonElement>getRequiredElement('sketch-eval-original');
    const variationButton = <HTMLButtonElement>getRequiredElement('sketch-eval-variation');
    const slideshowButton = <HTMLButtonElement>getRequiredElement('sketch-eval-slideshow');
    // const scriptElement = getRequiredElement('sketch-script');
    const canvasElement = <HTMLCanvasElement>getRequiredElement('sketch-canvas');
    // const canvasImageElement0 = getRequiredElement(IMG_0);
    const canvasImageElement1 = <HTMLImageElement>getRequiredElement(IMG_1);

    canvasImageElement1.addEventListener("animationend", animationEndListener1, false);
    setOpacity(IMG_1, 0);
    const shaders = await loadShaders([`${PREFIX}/shader/main-vert.glsl`,
                                       `${PREFIX}/shader/main-frag.glsl`,
                                       `${PREFIX}/shader/blit-vert.glsl`,
                                       `${PREFIX}/shader/blit-frag.glsl`]);
    gGLRenderer = new GLRenderer2(PREFIX, canvasElement, <IHashStrStr>shaders, g_render_texture_width, g_render_texture_height);
    //updateGlobalsFromURI();  // todo: 2020-06-25 re-enable

    // const script = scriptElement!.textContent;
    // const originalScript = script.slice();

    logDebug("init");

    await gGLRenderer.ensureTexture(TextureUnit.brushTexture, PREFIX, `brush.png`);
    await updateSketch(DisplayMode.snap);

    // gGLRenderer.loadTexture(`${PREFIX}/img/brush.png`)
    //   .then(async () => await updateSketch(DisplayMode.snap))
    //   .catch(error => console.error(error));

    originalButton.addEventListener('click', async () => {
        originalButton.disabled = true;

        gSeed = 0;
        updateToMode(SketchMode.normal);

        updateURIFromGlobals(true);

        await updateSketch(DisplayMode.fade);
    });

    slideshowButton.addEventListener('click', async () => {
        originalButton.disabled = false;

        if (updateToMode(SketchMode.slideshow)) {
            await updateSketch(DisplayMode.snap);
            const sketchImg1 = <HTMLImageElement>getRequiredElement(IMG_1);
            await gGLRenderer.copyImageDataTo(sketchImg1);

            // only call updateSketch if we're actually switching to SLIDESHOW mode as this will create a settimeout
            gTimeoutId = window.setTimeout(performSlideshow, 0);
        }
        updateURIFromGlobals(true);

    });

    variationButton.addEventListener('click', async () => {
        originalButton.disabled = false;

        const seedElement = <HTMLInputElement>getRequiredElement('sketch-seed');
        const newSeed = Math.random() * (1 << 30);
        seedElement.value = newSeed.toString();
        gSeed = getSeedValue(seedElement);

        updateToMode(SketchMode.normal);

        updateURIFromGlobals(true);

        await updateSketch(DisplayMode.fade);
    });

    window.addEventListener('popstate', async event => {
        updateGlobalsFromURI();
        await updateSketch(DisplayMode.snap);
    });

    canvasImageElement1.addEventListener('click', async () => {
        updateToMode(SketchMode.normal);

        updateURIFromGlobals(true);

        await updateSketch(DisplayMode.snap);
    });

    const escapeKey = 27;
    document.addEventListener('keydown', async event => {
        if (event.keyCode === escapeKey && gMode !== SketchMode.normal) {

            updateToMode(SketchMode.normal);

            updateURIFromGlobals(true);

            await updateSketch(DisplayMode.snap);

            event.preventDefault();
        }
    }, false);
}

document.addEventListener('DOMContentLoaded', () => {
    compatibilityHacks();
    main();
});


function downloadDialogHide() {
    const container = document.getElementById('download-dialog');
    if (container) {
        container.classList.add('hidden');
    }
}
