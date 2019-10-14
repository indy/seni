// --------------------------------------------------------------------------------
// jobTypes

const jobRender_1_Compile = 'RENDER_1_COMPILE';
const jobRender_2_ReceiveBitmapData = 'RENDER_2_RECEIVEBITMAPDATA';
const jobRender_3_RenderPackets = 'RENDER_3_RENDERPACKETS';
const jobUnparse = 'UNPARSE';
const jobBuildTraits = 'BUILD_TRAITS';
const jobInitialGeneration = 'INITIAL_GENERATION';
const jobNewGeneration = 'NEW_GENERATION';
const jobGenerateHelp = 'GENERATE_HELP';
const jobSingleGenotypeFromSeed = 'SINGLE_GENOTYPE_FROM_SEED';
const jobSimplifyScript = 'SIMPLIFY_SCRIPT';

// --------------------------------------------------------------------------------
// job

let numWorkers = 0;
const promiseWorkers = [];

class PromiseWorker {
  constructor(id, workerUrl) {
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
        log(`worker ${self.id} initialised`);
        return;
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

      if (type === jobRender_2_ReceiveBitmapData) {
        // ImageData is not a transferrable type
        self.worker.postMessage({ type, data });
      } else {
        self.worker.postMessage({ type, data });
      }
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

const Job = {
  request: async function(type, data, worker_id) {
    try {
      let worker = undefined;
      if (worker_id === undefined) {
        worker = await findAvailableWorker();
        log(`assigning ${type} to worker ${worker.getId()}`);
      } else {
        worker = promiseWorkers[worker_id];
        log(`explicitly assigning ${type} to worker ${worker.getId()}`);
      }

      const result = await worker.postMessage(type, data);
      log(`result ${type} id:${worker.getId()}`);

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
  },

  setup: function(numWorkersParam, path) {
    numWorkers = numWorkersParam;

    log(`workers::path = ${path}`);
    log(`workers::numWorkers = ${numWorkers}`);

    for (let i = 0; i < numWorkers; i++) {
      promiseWorkers[i] = new PromiseWorker(i, path);
    }
  },
};
