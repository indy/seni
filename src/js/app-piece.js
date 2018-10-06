/*
 *  Seni
 *  Copyright (C) 2018 Inderjit Gill <email@indy.io>
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

function renderBuffers(memory, buffers, w, h) {
  gGLRenderer.preDrawScene(w, h);

  const memoryF32 = new Float32Array(memory);

  buffers.forEach(buffer => {
    gGLRenderer.drawBuffer(memoryF32, buffer);
  });
}

function renderScript(config) {
  return Job.request(jobRender, config)
    .then(({ memory, buffers }) => {
      renderBuffers(memory, buffers, 500, 500);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
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
    console.log(`worker: error of ${error}`);
  });
}

export default function main() {
  const texturePathElement = getRequiredElement('piece-texture-path');
  const workerPathElement = getRequiredElement('piece-worker-path');

  Job.setup(2, workerPathElement.textContent);

  const originalButton = getRequiredElement('piece-eval-original');
  const evalButton = getRequiredElement('piece-eval');
  const scriptElement = getRequiredElement('piece-script');
  const canvasElement = getRequiredElement('piece-canvas');
  const seedElement = getRequiredElement('piece-seed');

  if (LOAD_FOR_SENI_APP_GALLERY === false) {
    // not really required, hack to load in other pieces
    const loadIdElement = getRequiredElement('piece-load-id');
    loadIdElement.addEventListener('change', event => {
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

  const scriptHash = Util.hashCode('whatever');

  gGLRenderer = new GLRenderer(canvasElement);

  const script = scriptElement.textContent;
  const originalScript = script.slice();
  showSimplifiedScript(script);

  gGLRenderer.loadTexture(texturePathElement.textContent)
    .then(() => renderScript({ script, scriptHash }))
    .catch(error => console.error(error));

  originalButton.addEventListener('click', () => {
    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    showSimplifiedScript(originalScript);
    originalButton.disabled = true;
  });

  evalButton.addEventListener('click', () => {
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
        console.log('fooked');
        console.error(error);
      });
  });
}
