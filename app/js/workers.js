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

import PromiseWorker from 'promise-worker';

const logToConsole = true;

let numWorkers = 8;
const promiseWorkers = [];
const working = [];

function findAvailableWorkerId() {
  return new Promise((resolve, _reject) => {
    setTimeout(function go() {
      let foundAvailableWorker = false;
      let id = 0;
      for (let i=0;i<numWorkers;i++) {
        if (working[i] === false) {
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
  working[workerId] = true;
  return promiseWorkers[workerId];
}

function releaseWorker(workerId) {
  working[workerId] = false;
}

function setup(numWorkersParam) {

  if (logToConsole) {
    console.log(`workers::numWorkers = ${numWorkersParam}`);
  }

  numWorkers = numWorkersParam;

  for (let i = 0; i < numWorkers; i++) {
    const w = new Worker(`/dist/worker.bundle.js`);
    promiseWorkers[i] = new PromiseWorker(w);
    working[i] = false;
  }
}

function perform(jobType, jobData) {

  return new Promise((resolve, reject) => {

    findAvailableWorkerId().then(id => {
      const worker = getWorker(id);

      const data = {
        type: jobType,
        workerId: id,
        data: jobData
      };

      return worker.postMessage(data);
    }).then(result => {

      releaseWorker(result.workerId);

      if (result.status === `OK`) {
        resolve(result.data);
      } else {
        reject(result.status);
      }
    }).catch(error => {
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
