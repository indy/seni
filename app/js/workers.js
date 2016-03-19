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

/*
 used on the main thread to manage the web workers
 */
import { jobRender } from './jobTypes';

const logToConsole = false;

let numWorkers = 0;
const promiseWorkers = [];

/*
function ab2str(arrayBuffer) {
  let res = '';
  const data = new Uint16Array(arrayBuffer);
  const len = data.byteLength / 2;
  for (let i = 0; i < len; i++) {
    res += String.fromCharCode(data[i]);
  }
  return res;
}
*/

class PromiseWorker {
  constructor(id, workerUrl) {
    const self = this;

    this.worker = new Worker(workerUrl);
    this.id = id;
    this.working = false;
    this.reject = undefined;
    this.resolve = undefined;

    this.worker.addEventListener('message', event => {
      // string data is always going to be in JSON formation
      // otherwise it will be a string encoded in an ArrayBuffer
      let error = undefined;
      let result = undefined;

      if (typeof(event.data) === 'string') {
        [error, result] = JSON.parse(event.data);
      } else {                  // ArrayBuffer
        // [error, result] = JSON.parse(ab2str(event.data));
        [error, result] = event.data;
        return self.resolve(result);
      }

      if (error) {
        return self.reject(new Error(error.message));
      }
      return self.resolve(result);
    });
  }

  postMessage(type, data) {
    const self = this;

    return new Promise((resolve, reject) => {
      self.resolve = resolve;
      self.reject = reject;
      self.worker.postMessage(JSON.stringify({ type, data }));
    });
  }

  employ() {
    this.working = true;
    return this;
  }

  release() {
    this.working = false;
    return this;
  }

  isWorking() {
    return this.working;
  }

  getId() {
    return this.id;
  }
}

function setup(numWorkersParam) {
  if (logToConsole) {
    console.log(`workers::numWorkers = ${numWorkersParam}`);
  }

  numWorkers = numWorkersParam;
  for (let i = 0; i < numWorkers; i++) {
    promiseWorkers[i] = new PromiseWorker(i, '/dist/worker.bundle.js');
  }
}

function findAvailableWorker() {
  return new Promise((resolve, _reject) => {
    setTimeout(function go() {
      for (let i=0;i<numWorkers;i++) {
        if (promiseWorkers[i].isWorking() === false) {
          resolve(promiseWorkers[i].employ());
          return;
        }
      }
      // todo?: reject if waiting too long?
      setTimeout(go, 100);
    });
  });
}

function perform(type, data) {
  return new Promise((resolve, reject) => {
    let worker = undefined;

    let preBefore = 0;
    let postAfter = 0;

    findAvailableWorker().then(worker_ => {
      if (type === jobRender) {
        preBefore = performance.now();
      }

      worker = worker_;

      if (logToConsole) {
        console.log(`assigning ${type} to worker ${worker.getId()}`);
      }

      return worker.postMessage(type, data);
    }).then(result => {
      if (type === jobRender) {
        postAfter = performance.now();

        const before = result.before;
        const after = result.after;

        const sendTime = before - preBefore;
        const processTime = after - before;
        const receiveTime = postAfter - after;

        const res = Object.assign({sendTime, processTime, receiveTime}, result);

        worker.release();
        resolve(res);
        return;
      }

      if (logToConsole) {
        console.log(`result ${type} id:${worker.getId()}`);
      }
      worker.release();
      resolve(result);
    }).catch(error => {
      if (worker !== undefined) {
        worker.release();
      }
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

export default {
  setup,
  perform
};
