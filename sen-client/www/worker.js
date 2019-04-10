/*
 *  Seni
 *  Copyright (C) 2019 Inderjit Gill <email@indy.io>
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


(function() {
    var wasm;
    const __exports = {};


    let cachedTextDecoder = new TextDecoder('utf-8');

    let cachegetUint8Memory = null;
    function getUint8Memory() {
        if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
            cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
        }
        return cachegetUint8Memory;
    }

    function getStringFromWasm(ptr, len) {
        return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
    }

    __exports.__wbg_log_71c040d88e754893 = function(arg0, arg1) {
        let varg0 = getStringFromWasm(arg0, arg1);
        console.log(varg0);
    };

    let cachedGlobalArgumentPtr = null;
    function globalArgumentPtr() {
        if (cachedGlobalArgumentPtr === null) {
            cachedGlobalArgumentPtr = wasm.__wbindgen_global_argument_ptr();
        }
        return cachedGlobalArgumentPtr;
    }

    let cachegetUint32Memory = null;
    function getUint32Memory() {
        if (cachegetUint32Memory === null || cachegetUint32Memory.buffer !== wasm.memory.buffer) {
            cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
        }
        return cachegetUint32Memory;
    }

    let cachedTextEncoder = new TextEncoder('utf-8');

    let WASM_VECTOR_LEN = 0;

    function passStringToWasm(arg) {

        const buf = cachedTextEncoder.encode(arg);
        const ptr = wasm.__wbindgen_malloc(buf.length);
        getUint8Memory().set(buf, ptr);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    function freeBridge(ptr) {

        wasm.__wbg_bridge_free(ptr);
    }
    /**
    */
    class Bridge {

        free() {
            const ptr = this.ptr;
            this.ptr = 0;
            freeBridge(ptr);
        }

        /**
        * @returns {}
        */
        constructor() {
            this.ptr = wasm.bridge_new();
        }
        /**
        * @returns {string}
        */
        get_genotype_buffer_string() {
            const retptr = globalArgumentPtr();
            wasm.bridge_get_genotype_buffer_string(retptr, this.ptr);
            const mem = getUint32Memory();
            const rustptr = mem[retptr / 4];
            const rustlen = mem[retptr / 4 + 1];

            const realRet = getStringFromWasm(rustptr, rustlen).slice();
            wasm.__wbindgen_free(rustptr, rustlen * 1);
            return realRet;

        }
        /**
        * @param {string} arg0
        * @returns {void}
        */
        set_genotype_buffer_string(arg0) {
            const ptr0 = passStringToWasm(arg0);
            const len0 = WASM_VECTOR_LEN;
            try {
                return wasm.bridge_set_genotype_buffer_string(this.ptr, ptr0, len0);

            } finally {
                wasm.__wbindgen_free(ptr0, len0 * 1);

            }

        }
        /**
        * @returns {string}
        */
        get_traits_buffer_string() {
            const retptr = globalArgumentPtr();
            wasm.bridge_get_traits_buffer_string(retptr, this.ptr);
            const mem = getUint32Memory();
            const rustptr = mem[retptr / 4];
            const rustlen = mem[retptr / 4 + 1];

            const realRet = getStringFromWasm(rustptr, rustlen).slice();
            wasm.__wbindgen_free(rustptr, rustlen * 1);
            return realRet;

        }
        /**
        * @param {string} arg0
        * @returns {void}
        */
        set_traits_buffer_string(arg0) {
            const ptr0 = passStringToWasm(arg0);
            const len0 = WASM_VECTOR_LEN;
            try {
                return wasm.bridge_set_traits_buffer_string(this.ptr, ptr0, len0);

            } finally {
                wasm.__wbindgen_free(ptr0, len0 * 1);

            }

        }
        /**
        * @returns {string}
        */
        get_out_source_buffer_string() {
            const retptr = globalArgumentPtr();
            wasm.bridge_get_out_source_buffer_string(retptr, this.ptr);
            const mem = getUint32Memory();
            const rustptr = mem[retptr / 4];
            const rustlen = mem[retptr / 4 + 1];

            const realRet = getStringFromWasm(rustptr, rustlen).slice();
            wasm.__wbindgen_free(rustptr, rustlen * 1);
            return realRet;

        }
        /**
        * @param {string} arg0
        * @returns {void}
        */
        set_out_source_buffer_string(arg0) {
            const ptr0 = passStringToWasm(arg0);
            const len0 = WASM_VECTOR_LEN;
            try {
                return wasm.bridge_set_out_source_buffer_string(this.ptr, ptr0, len0);

            } finally {
                wasm.__wbindgen_free(ptr0, len0 * 1);

            }

        }
        /**
        * @returns {string}
        */
        get_source_buffer_string() {
            const retptr = globalArgumentPtr();
            wasm.bridge_get_source_buffer_string(retptr, this.ptr);
            const mem = getUint32Memory();
            const rustptr = mem[retptr / 4];
            const rustlen = mem[retptr / 4 + 1];

            const realRet = getStringFromWasm(rustptr, rustlen).slice();
            wasm.__wbindgen_free(rustptr, rustlen * 1);
            return realRet;

        }
        /**
        * @param {string} arg0
        * @returns {void}
        */
        set_source_buffer_string(arg0) {
            const ptr0 = passStringToWasm(arg0);
            const len0 = WASM_VECTOR_LEN;
            try {
                return wasm.bridge_set_source_buffer_string(this.ptr, ptr0, len0);

            } finally {
                wasm.__wbindgen_free(ptr0, len0 * 1);

            }

        }
        /**
        * @returns {void}
        */
        sen_startup() {
            return wasm.bridge_sen_startup(this.ptr);
        }
        /**
        * @returns {void}
        */
        sen_shutdown() {
            return wasm.bridge_sen_shutdown(this.ptr);
        }
        /**
        * @returns {number}
        */
        compile_to_render_packets() {
            return wasm.bridge_compile_to_render_packets(this.ptr);
        }
        /**
        * @param {number} arg0
        * @returns {number}
        */
        get_render_packet_geo_len(arg0) {
            return wasm.bridge_get_render_packet_geo_len(this.ptr, arg0);
        }
        /**
        * @param {number} arg0
        * @returns {number}
        */
        get_render_packet_geo_ptr(arg0) {
            return wasm.bridge_get_render_packet_geo_ptr(this.ptr, arg0);
        }
        /**
        * @returns {boolean}
        */
        build_traits() {
            return (wasm.bridge_build_traits(this.ptr)) !== 0;
        }
        /**
        * @param {number} arg0
        * @returns {boolean}
        */
        single_genotype_from_seed(arg0) {
            return (wasm.bridge_single_genotype_from_seed(this.ptr, arg0)) !== 0;
        }
        /**
        * @param {number} arg0
        * @param {number} arg1
        * @returns {boolean}
        */
        create_initial_generation(arg0, arg1) {
            return (wasm.bridge_create_initial_generation(this.ptr, arg0, arg1)) !== 0;
        }
        /**
        * @param {number} arg0
        * @returns {boolean}
        */
        genotype_move_to_buffer(arg0) {
            return (wasm.bridge_genotype_move_to_buffer(this.ptr, arg0)) !== 0;
        }
        /**
        * @returns {void}
        */
        script_cleanup() {
            return wasm.bridge_script_cleanup(this.ptr);
        }
        /**
        * @param {boolean} arg0
        * @returns {void}
        */
        use_genotype_when_compiling(arg0) {
            return wasm.bridge_use_genotype_when_compiling(this.ptr, arg0);
        }
        /**
        * @returns {void}
        */
        next_generation_prepare() {
            return wasm.bridge_next_generation_prepare(this.ptr);
        }
        /**
        * @returns {void}
        */
        next_generation_add_genotype() {
            return wasm.bridge_next_generation_add_genotype(this.ptr);
        }
        /**
        * @param {number} arg0
        * @param {number} arg1
        * @param {number} arg2
        * @param {number} arg3
        * @returns {boolean}
        */
        next_generation_build(arg0, arg1, arg2, arg3) {
            return (wasm.bridge_next_generation_build(this.ptr, arg0, arg1, arg2, arg3)) !== 0;
        }
        /**
        * @returns {void}
        */
        unparse_with_genotype() {
            return wasm.bridge_unparse_with_genotype(this.ptr);
        }
        /**
        * @returns {void}
        */
        simplify_script() {
            return wasm.bridge_simplify_script(this.ptr);
        }
    }
    __exports.Bridge = Bridge;

    // ISG HACK
    __exports.wasm = wasm;

  __exports.__wbindgen_throw = function(ptr, len) {
        throw new Error(getStringFromWasm(ptr, len));
    };

    function init(path_or_module) {
        let instantiation;
        const imports = { './sen_client': __exports };
        if (path_or_module instanceof WebAssembly.Module) {
            instantiation = WebAssembly.instantiate(path_or_module, imports)
            .then(instance => {
            return { instance, module: path_or_module }
        });
    } else {
        const data = fetch(path_or_module);
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            instantiation = WebAssembly.instantiateStreaming(data, imports);
        } else {
            instantiation = data
            .then(response => response.arrayBuffer())
            .then(buffer => WebAssembly.instantiate(buffer, imports));
        }
    }
    return instantiation.then(({instance}) => {
        wasm = init.wasm = instance.exports;

    });
};
self.wasm_bindgen = Object.assign(init, __exports);
})();


/*
  // copy this line into the wasm_bindgen object above whenever it's regenerated

    // ISG HACK
    __exports.wasm = wasm;

*/

// global state
let gState = {};

const jobRender = 'RENDER';
const jobUnparse = 'UNPARSE';
const jobBuildTraits = 'BUILD_TRAITS';
const jobInitialGeneration = 'INITIAL_GENERATION';
const jobNewGeneration = 'NEW_GENERATION';
const jobGenerateHelp = 'GENERATE_HELP';
const jobSingleGenotypeFromSeed = 'SINGLE_GENOTYPE_FROM_SEED';
const jobSimplifyScript = 'SIMPLIFY_SCRIPT';

class KonsoleProxy {
  constructor() {
    this.messages = [];
  }

  clear() {
    this.messages = [];
  }

  log(msg) {
    this.messages.push(msg);
  }

  collectMessages() {
    return this.messages.join('\n');
  }
}

const konsoleProxy = new KonsoleProxy();


/*
  function pointerToFloat32Array(ptr, length) {
  const nByte = 4;
  const pos = ptr / nByte;
  return SeniWasm.instance.memory.F32.subarray(pos, pos + length);
  }

  function pointerToArrayBufferCopy(ptr, length) {
  const nByte = 4;
  const pos = ptr / nByte;
  return SeniWasm.instance.memory.F32.slice(pos, pos + length);
  }
*/

function render({ script /*, scriptHash*/, genotype }) {
  konsoleProxy.clear();
  const buffers = [];

  if (genotype) {
    // console.log("render: using a genotype");
    gState.bridge.use_genotype_when_compiling(true);
    gState.bridge.set_genotype_buffer_string(genotype);
  } else {
    // console.log("render: not using a genotype");
    gState.bridge.use_genotype_when_compiling(false);
  }

  // need to setString before calling compileToRenderPackets
  gState.bridge.set_source_buffer_string(script);
  const numRenderPackets = gState.bridge.compile_to_render_packets();
  // konsoleProxy.log(`numRenderPackets = ${numRenderPackets}`);

  for (let i = 0; i < numRenderPackets; i++) {
    const buffer = {};
    buffer.geo_len = gState.bridge.get_render_packet_geo_len(i);
    if (buffer.geo_len > 0) {
      buffer.geo_ptr = gState.bridge.get_render_packet_geo_ptr(i);
      buffers.push(buffer);
    }

    // const numVertices = gState.bridge.get_render_packet_num_vertices(i);
    // // konsoleProxy.log(`render_packet ${i}: numVertices = ${numVertices}`);

    // if (numVertices > 0) {
    //   const buffer = {};

      // buffer.vbufAddress = gState.bridge.get_render_packet_vBuf(i);
      // buffer.cbufAddress = gState.bridge.get_render_packet_cBuf(i);
      // buffer.tbufAddress = gState.bridge.get_render_packet_tBuf(i);

      // buffer.numVertices = numVertices;

    //   buffers.push(buffer);
    // }
  }

  gState.bridge.script_cleanup();

  const logMessages = konsoleProxy.collectMessages();
  const title = '';

  // make a copy of the wasm memory
  //
  // note: (05-12-2017) required by Firefox as that doesn't allow transferring
  // Wasm ArrayBuffers to different threads
  // (errors with: cannot transfer WebAssembly/asm.js ArrayBuffer)
  //
  // WTF note: Expected a perfomance cost in Chrome due to the slice operation
  // but it seemed to either have no effect or to make the rendering faster!?!
  //
  const wasmMemory = gState.memory.buffer;
  const memory = wasmMemory.slice();

  return [{ logMessages }, { title, memory, buffers }];
}

function unparse({ script/*, scriptHash*/, genotype }) {
  konsoleProxy.clear();

  // console.log(`genotype is ${genotype}`);
  // console.log(`script is ${script}`);

  gState.bridge.set_source_buffer_string(script);
  gState.bridge.set_genotype_buffer_string(genotype);

  gState.bridge.unparse_with_genotype();
  const newScript = gState.bridge.get_out_source_buffer_string();

  // console.log(`new script: ${newScript}`);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { script: newScript }];
}

function buildTraits({ script /*, scriptHash */ }) {
  konsoleProxy.clear();

  gState.bridge.set_source_buffer_string(script);

  let traits = [];
  const numTraits = gState.bridge.build_traits();
  const validTraits = numTraits !== -1;

  if (validTraits) {
    traits = gState.bridge.get_traits_buffer_string();
    // console.log(traits);
  }

  const logMessages = konsoleProxy.collectMessages();
  return [{ logMessages }, { validTraits, traits }];
}

// transfers the contents of g_genotype_list from the wasm side
function getGenotypesFromWasm(populationSize) {
  const genotypes = [];
  let s;

  for (let i = 0; i < populationSize; i++) {
    gState.bridge.genotype_move_to_buffer(i);
    s = gState.bridge.get_genotype_buffer_string();
    genotypes.push(s);
  }

  return genotypes;
}

function createInitialGeneration({ populationSize, traits }) {
  konsoleProxy.clear();

  // console.log("createInitialGeneration: using traits:");
  // console.log(traits);

  gState.bridge.set_traits_buffer_string(traits);

  const seed = Math.floor(Math.random() * 1024);
  // konsoleProxy.log(`createInitialGeneration seed: ${seed}`);
  // konsoleProxy.log(`createInitialGeneration populationSize: ${populationSize}`);

  gState.bridge.create_initial_generation(populationSize, seed);

  const genotypes = getGenotypesFromWasm(populationSize);
  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { genotypes }];
}

function singleGenotypeFromSeed({ seed, traits }) {
  konsoleProxy.clear();

  gState.bridge.set_traits_buffer_string(traits);

  // konsoleProxy.log(`singleGenotypeFromSeed seed: ${seed}`);

  gState.bridge.single_genotype_from_seed(seed);

  const genotypes = getGenotypesFromWasm(1);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { genotype: genotypes[0] }];
}

function simplifyScript({ script }) {
  konsoleProxy.clear();

  gState.bridge.set_source_buffer_string(script);

  gState.bridge.simplify_script();

  const newScript = gState.bridge.get_out_source_buffer_string();

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { script: newScript }];
}

function newGeneration({genotypes, populationSize, traits, mutationRate, rng}) {
  konsoleProxy.clear();

  gState.bridge.next_generation_prepare();
  for (let i = 0; i < genotypes.length; i++) {
    gState.bridge.set_genotype_buffer_string(genotypes[i]);
    gState.bridge.next_generation_add_genotype();
  }

  gState.bridge.set_traits_buffer_string(traits);
  gState.bridge.next_generation_build(genotypes.length, populationSize,
                                      mutationRate, rng);

  const newGenotypes = getGenotypesFromWasm(populationSize);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { genotypes: newGenotypes }];
}

const options = {
  imports: {
    performance_now() {
      return performance.now();
    }
  }
};


/*
  function freeModule() {

  Module._free(gState.bridge.ptr);
  Module._free(gState.bridge.vbuf);
  Module._free(gState.bridge.cbuf);
  Module._free(gState.bridge.tbuf);

  gState.bridge.senShutdown();
  }
*/

function messageHandler(type, data) {
  switch (type) {
  case jobRender:
    // console.log("jobRender");
    return render(data);
  case jobUnparse:
    // console.log("jobUnparse");
    return unparse(data);
  case jobBuildTraits:
    // console.log("jobBuildTraits");
    return buildTraits(data);
  case jobInitialGeneration:
    // console.log("jobInitialGeneration");
    return createInitialGeneration(data);
  case jobSingleGenotypeFromSeed:
    // console.log("jobSingleGenotypeFromSeed");
    return singleGenotypeFromSeed(data);
  case jobSimplifyScript:
    // console.log("jobSimplifyScript");
    return simplifyScript(data);
  case jobNewGeneration:
    // console.log("jobNewGeneration");
    return newGeneration(data);
  default:
    // throw unknown type
    throw new Error(`worker.js: Unknown type: ${type}`);
  }
}

/*
  postMessage will always return an array of two items: [status, result]

  status = {
  error: { message: "something fucked up" }
  systemInitialised: true
  logMessages: []
  }
*/

addEventListener('message', e => {
  try {
    const { type, data } = JSON.parse(e.data);

    const [status, result] = messageHandler(type, data);

    if (type === jobRender) {
      const transferrable = [];

      if (result.buffers && result.buffers.length > 0) {
        transferrable.push(result.memory);
      }

      postMessage([status, result], transferrable);
    } else {
      const sendData = JSON.stringify([status, result]);
      // console.log(`worker.js:sendData = ${sendData}`);
      postMessage(sendData);
    }
  } catch (error) {
    postMessage(JSON.stringify([{error: {message: error.message}}, undefined]));
  }
});

wasm_bindgen('./sen_client_bg.wasm')
  .then(() => {
    // hack to access the memory
    // the build.sh has a sed command to export the wasm object
    // replace the js renderer with a rust implmentation to get rid of this hack
    // memory = wasm_bindgen.wasm.memory;
    const { Bridge } = wasm_bindgen;
    gState.bridge = new Bridge();
    gState.memory = wasm_bindgen.wasm.memory;

    gState.bridge.sen_startup();

    // send the job system an initialised message so
    // that it can start sending jobs to this worker
    const sendData = JSON.stringify([{systemInitialised: true}]);
    postMessage(sendData);

  })
  .catch(console.error);
