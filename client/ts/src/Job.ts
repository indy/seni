/*
 *  Seni
 *  Copyright (C) 2020 Inderjit Gill <email@indy.io>
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

///<reference path='Log.ts'/>
///<reference path='JobType.ts'/>

namespace Job {
    let numWorkers: number = 0;
    const promiseWorkers: Array<PromiseWorker> = [];

    function findAvailableWorker(): Promise<PromiseWorker> {
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

    export async function request(type: JobType, data: any, worker_id: number | undefined) {
        try {
            let worker = undefined;
            if (worker_id === undefined) {
                worker = await findAvailableWorker();
                Log.log(`assigning ${type} to worker ${worker.getId()}`);
            } else {
                worker = promiseWorkers[worker_id];
                Log.log(`explicitly assigning ${type} to worker ${worker.getId()}`);
            }

            const result = await worker.postMessage(type, data);
            Log.log(`result ${type} id:${worker.getId()}`);

            if(!data.__retain) {
                worker.release();
            }

            result.__worker_id = worker.getId();

            return result;
        } catch (error) {
            // handle error
            console.error(`worker (job:${type}): error of ${error}`);
            return undefined;         // ???
        }
    }

    export function setup(numWorkersParam: number, path: string) {
        numWorkers = numWorkersParam;

        Log.log(`workers::path = ${path}`);
        Log.log(`workers::numWorkers = ${numWorkers}`);

        for (let i = 0; i < numWorkers; i++) {
            promiseWorkers[i] = new PromiseWorker(i, path);
        }
    }
}

class PromiseWorker {
    worker: Worker;
    id: number;
    initialised: boolean;
    working: boolean;
    reject: any;
    resolve: any;

    constructor(id: number, workerUrl: string) {
        const self = this;

        // <2019-04-13 Sat>
        // would be good to use module syntax in the worker.js file.
        // this would enable a more modern way of instantiating the wasm module
        // see https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html?highlight=export,memory#without-a-bundler
        //
        // This should be enabled with:
        // this.worker = new Worker(workerUrl, {type:'module'});
        //
        // unfortunately there is a bug in Chromium preventing this:
        // https://bugs.chromium.org/p/chromium/issues/detail?id=680046
        // original info from:
        // https://www.codedread.com/blog/archives/2017/10/19/web-workers-can-be-es6-modules-too/

        this.worker = new Worker(workerUrl);
        this.id = id;
        this.initialised = false; // true when the worker has loaded it's wasm file
        this.working = false;
        this.reject = undefined;
        this.resolve = undefined;

        this.worker.addEventListener('message', event => {

            const [status, result] = event.data;

            if (status.systemInitialised) {
                self.initialised = true;
                Log.log(`worker ${self.id} initialised`);
                return;
            }

            if (status.error) {
                self.reject(new Error(status.error.message));
            } else {
                self.resolve(result);
            }
        });
    }

    postMessage(type: JobType, data: any): Promise<any> {
        const self = this;

        return new Promise((resolve, reject) => {
            self.resolve = resolve;
            self.reject = reject;

            if (type === JobType.jobRender_2_ReceiveBitmapData) {
                // ImageData is not a transferrable type
                self.worker.postMessage({ type, data });
            } else {
                self.worker.postMessage({ type, data });
            }
        });
    }

    employ(): PromiseWorker {
        this.working = true;
        return this;
    }

    release(): PromiseWorker {
        this.working = false;
        return this;
    }

    isInitialised(): boolean {
        return this.initialised;
    }

    isWorking(): boolean {
        return this.working;
    }

    getId(): number {
        return this.id;
    }
}
