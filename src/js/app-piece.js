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
    .then(({ title, memory, buffers }) => {
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

export default function main() {
  Job.setup(2);

  const textArea = document.getElementById('edit-textarea');
  const canvasElement = document.getElementById('render-canvas');

  const script = textArea.value;
  const scriptHash = Util.hashCode(script);

  const button = document.getElementById('eval-btn');
  button.addEventListener('click', event => {

    buildTraits({ script, scriptHash })
      .then(({ traits }) => buildGenotype({ traits, seed: 3 }))
      .then(({ genotype }) => renderScript({ script, scriptHash, genotype }))
      .catch(error => {
        console.log('fooked');
        console.error(error);
      });
  });

  gGLRenderer = new GLRenderer(canvasElement);
  gGLRenderer.loadTexture('/img/texture.png')
    .then(() => {
      console.log('loaded texture');
      renderScript({ script, scriptHash });
    })
    .catch(error => console.error(error));

}
