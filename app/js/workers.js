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

const logToConsole = true;

let numWorkers = 8;
const promiseWorkers = [];

class PromiseWorker {
  constructor(workerUrl) {
    const self = this;

    this.worker = new Worker(workerUrl);
    this.working = false;
    this.reject = undefined;
    this.resolve = undefined;

    this.worker.addEventListener('message', event => {
      const [error, result] = JSON.parse(event.data);

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

  setWorking(value) {
    this.working = value;
  }

  getWorking() {
    return this.working;
  }
}

function setup(numWorkersParam) {
  if (logToConsole) {
    console.log(`workers::numWorkers = ${numWorkersParam}`);
  }

  numWorkers = numWorkersParam;
  for (let i = 0; i < numWorkers; i++) {
    promiseWorkers[i] = new PromiseWorker('/dist/worker.bundle.js');
  }
}

function findAvailableWorkerId() {
  return new Promise((resolve, _reject) => {
    setTimeout(function go() {
      let foundAvailableWorker = false;
      let id = 0;
      for (let i=0;i<numWorkers;i++) {
        if (promiseWorkers[i].getWorking() === false) {
          foundAvailableWorker = true;
          id = i;
          break;
        }
      }

      // todo?: reject if waiting too long?

      if (foundAvailableWorker) {
        resolve(id);
      } else {
        setTimeout(go, 500);
      }
    });
  });
}

function getWorker(workerId) {
  promiseWorkers[workerId].setWorking(true);
  return promiseWorkers[workerId];
}

function releaseWorker(workerId) {
  promiseWorkers[workerId].setWorking(false);
}

function isValidWorkerId(workerId) {
  return workerId >= 0 && workerId < numWorkers;
}

function perform(type, data) {
  return new Promise((resolve, reject) => {
    let workerId = undefined;

    findAvailableWorkerId().then(id => {
      const worker = getWorker(id);
      workerId = id;

      if (logToConsole) {
        console.log(`assigning ${type} to worker ${id}`);
      }

      return worker.postMessage(type, data);
    }).then(result => {
      if (logToConsole) {
        console.log(`result ${type} id:${workerId}`);
      }

      releaseWorker(workerId);
      resolve(result);
    }).catch(error => {
      if (isValidWorkerId(workerId)) {
        releaseWorker(workerId);
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
