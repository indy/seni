/*
 *  Senie
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

import GLRenderer from './senie/GLRenderer';
import Util from './senie/Util';
import Job from './job';
import { jobRender,
         jobUnparse,
         jobBuildTraits,
         jobSingleGenotypeFromSeed
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


export default function main() {
  Job.setup(2);

  const originalButton = document.getElementById('piece-eval-original');
  const evalButton = document.getElementById('piece-eval');
  const scriptElement = document.getElementById('piece-script');
  const canvasElement = document.getElementById('piece-canvas');
  const seedElement = document.getElementById('piece-seed');
  const script = scriptElement.textContent;
  const scriptHash = Util.hashCode(script);
  const originalScript = script.slice();

  originalButton.addEventListener('click', () => {
    renderScript({ script: originalScript, scriptHash });
    scriptElement.textContent = originalScript;
    originalButton.disabled = true;
  });

  evalButton.addEventListener('click', () => {
    originalButton.disabled = false;
    const seedValue = getSeedValue(seedElement);
    buildTraits({ script, scriptHash })
      .then(({ traits }) => buildGenotype({ traits, seed: seedValue }))
      .then(({ genotype }) => {
        const config = { script, scriptHash };
        if (seedValue !== 0) {
          config.genotype = genotype;
        }
        renderScript(config);

        return unparse({ script, genotype });
      })
      .then(({ script }) => {
        scriptElement.textContent = script;
      })
      .catch(error => {
        console.log('fooked');
        console.error(error);
      });

    const newSeed = Math.random() * (1 << 30);
    seedElement.value = parseInt(newSeed, 10);
  });

  gGLRenderer = new GLRenderer(canvasElement);
  gGLRenderer.loadTexture('/img/texture.png')
    .then(() => {
      renderScript({ script, scriptHash });
    })
    .catch(error => console.error(error));
}
