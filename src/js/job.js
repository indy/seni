/*
 *  Sen
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

const logToConsole = false;

let numWorkers = 0;
const promiseWorkers = [];

class PromiseWorker {
  constructor(id, workerUrl) {
    const self = this;

    this.worker = new Worker(workerUrl);
    this.id = id;
    this.initialised = false; // true when the worker has loaded it's wasm file
    this.working = false;
    this.reject = undefined;
    this.resolve = undefined;

    this.worker.addEventListener('message', event => {
      // string data is always going to be in JSON formation
      // otherwise it will be a string encoded in an ArrayBuffer
      let status = undefined;
      let result = undefined;

      if (typeof(event.data) === 'string') {
        [status, result] = JSON.parse(event.data);

        if (status.systemInitialised) {
          self.initialised = true;
          return;
        }

      } else {                  // ArrayBuffer
        [status, result] = event.data;
      }

      if (status.logMessages && status.logMessages.length > 0) {
        console.log(status.logMessages);
      }

      if (status.error) {
        self.reject(new Error(status.error.message));
      } else {
        self.resolve(result);
      }
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

  isInitialised() {
    return this.initialised;
  }

  isWorking() {
    return this.working;
  }

  getId() {
    return this.id;
  }
}

function setup(numWorkersParam, path) {
  numWorkers = numWorkersParam;

  if (logToConsole) {
    console.log(`workers::path = ${path}`);
    console.log(`workers::numWorkers = ${numWorkers}`);
  }

  for (let i = 0; i < numWorkers; i++) {
    promiseWorkers[i] = new PromiseWorker(i, path);
  }
}

function findAvailableWorker() {
  return new Promise((resolve, _reject) => {
    setTimeout(function go() {
      for (let i=0;i<numWorkers;i++) {
        if (promiseWorkers[i].isInitialised() === true &&
            promiseWorkers[i].isWorking() === false) {
          resolve(promiseWorkers[i].employ());
          return;
        }
      }
      // todo?: reject if waiting too long?
      setTimeout(go, 100);
    });
  });
}

function request(type, data) {
  return new Promise((resolve, reject) => {
    let worker = undefined;

    findAvailableWorker().then(worker_ => {
      worker = worker_;
      if (logToConsole) {
        console.log(`assigning ${type} to worker ${worker.getId()}`);
      }
      return worker.postMessage(type, data);
    }).then(result => {
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
  request
};
