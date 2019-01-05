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


// THIS IS A PLACEHOLDER FILE



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

    __exports.__wbg_log_00db1e67318cc0fd = function(arg0, arg1) {
        let varg0 = getStringFromWasm(arg0, arg1);
        console.log(varg0);
    };
    /**
    * @returns {void}
    */
    __exports.say_hi = function() {
        return wasm.say_hi();
    };

    /**
    * @returns {void}
    */
    __exports.lenlen = function() {
        return wasm.lenlen();
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

const { say_hi, lenlen } = wasm_bindgen;


const jobRender = 'RENDER';
const jobUnparse = 'UNPARSE';
const jobBuildTraits = 'BUILD_TRAITS';
const jobInitialGeneration = 'INITIAL_GENERATION';
const jobNewGeneration = 'NEW_GENERATION';
const jobGenerateHelp = 'GENERATE_HELP';
const jobSingleGenotypeFromSeed = 'SINGLE_GENOTYPE_FROM_SEED';
const jobSimplifyScript = 'SIMPLIFY_SCRIPT';

const SeniWasm = {};

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------


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
    // console.log(`renderWasm genotype: ${genotype}`);
    SeniWasm.useGenotypeWhenCompiling(true);
    SeniWasm.setString(SeniWasm.genotype_buffer, genotype);
  } else {
    SeniWasm.useGenotypeWhenCompiling(false);
  }

  // need to setString before calling compileToRenderPackets
  SeniWasm.setString(SeniWasm.source_buffer, script);
  const numRenderPackets = SeniWasm.compileToRenderPackets();
  // konsoleProxy.log(`numRenderPackets = ${numRenderPackets}`);

  for (let i = 0; i < numRenderPackets; i++) {
    const numVertices = SeniWasm.getRenderPacketNumVertices(i);
    // konsoleProxy.log(`render_packet ${i}: numVertices = ${numVertices}`);

    if (numVertices > 0) {
      const buffer = {};

      buffer.vbufAddress = SeniWasm.getRenderPacketVBuf(i);
      buffer.cbufAddress = SeniWasm.getRenderPacketCBuf(i);
      buffer.tbufAddress = SeniWasm.getRenderPacketTBuf(i);

      buffer.numVertices = numVertices;
      buffers.push(buffer);
    }
  }

  SeniWasm.scriptCleanup();

  const logMessages = konsoleProxy.collectMessages();
  const title = 'WASM woohoo';

  // make a copy of the wasm memory
  //
  // note: (05-12-2017) required by Firefox as that doesn't allow transferring
  // Wasm ArrayBuffers to different threads
  // (errors with: cannot transfer WebAssembly/asm.js ArrayBuffer)
  //
  // WTF note: Expected a perfomance cost in Chrome due to the slice operation
  // but it seemed to either have no effect or to make the rendering faster!?!
  //
  const wasmMemory = SeniWasm.instance.memory.buffer;
  const memory = wasmMemory.slice();

  return [{ logMessages }, { title, memory, buffers }];
}

function unparse({ script/*, scriptHash*/, genotype }) {
  konsoleProxy.clear();

  // console.log(`genotype is ${genotype}`);
  // console.log(`script is ${script}`);

  SeniWasm.setString(SeniWasm.source_buffer, script);
  SeniWasm.setString(SeniWasm.genotype_buffer, genotype);

  SeniWasm.unparseWithGenotype();
  const newScript = SeniWasm.getString(SeniWasm.out_source_buffer);

  // console.log(`new script: ${newScript}`);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { script: newScript }];
}

function buildTraits({ script /*, scriptHash */ }) {
  konsoleProxy.clear();

  SeniWasm.setString(SeniWasm.source_buffer, script);

  let traits = [];
  const numTraits = SeniWasm.buildTraits();
  const validTraits = numTraits !== -1;

  if (validTraits) {
    traits = SeniWasm.getString(SeniWasm.traits_buffer);
  }

  const logMessages = konsoleProxy.collectMessages();
  return [{ logMessages }, { validTraits, traits }];
}

// transfers the contents of g_genotype_list from the wasm side
function getGenotypesFromWasm(populationSize) {
  const genotypes = [];
  let s;

  for (let i = 0; i < populationSize; i++) {
    SeniWasm.genotypeMoveToBuffer(i);
    s = SeniWasm.getString(SeniWasm.genotype_buffer);
    genotypes.push(s);
  }

  return genotypes;
}

function createInitialGeneration({ populationSize, traits }) {
  konsoleProxy.clear();

  SeniWasm.setString(SeniWasm.traits_buffer, traits);

  const seed = Math.floor(Math.random() * 1024);
  // konsoleProxy.log(`createInitialGeneration seed: ${seed}`);
  // konsoleProxy.log(`createInitialGeneration populationSize: ${populationSize}`);

  SeniWasm.createInitialGeneration(populationSize, seed);

  const genotypes = getGenotypesFromWasm(populationSize);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { genotypes }];
}

function singleGenotypeFromSeed({ seed, traits }) {
  konsoleProxy.clear();

  SeniWasm.setString(SeniWasm.traits_buffer, traits);

  // konsoleProxy.log(`singleGenotypeFromSeed seed: ${seed}`);

  SeniWasm.singleGenotypeFromSeed(seed);

  const genotypes = getGenotypesFromWasm(1);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { genotype: genotypes[0] }];
}

function simplifyScript({ script }) {
  konsoleProxy.clear();

  SeniWasm.setString(SeniWasm.source_buffer, script);

  SeniWasm.simplifyScript();

  const newScript = SeniWasm.getString(SeniWasm.out_source_buffer);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { script: newScript }];
}

function newGeneration({genotypes, populationSize, traits, mutationRate, rng}) {
  konsoleProxy.clear();

  SeniWasm.nextGenerationPrepare();
  for (let i = 0; i < genotypes.length; i++) {
    SeniWasm.setString(SeniWasm.genotype_buffer, genotypes[i]);
    SeniWasm.nextGenerationAddGenotype();
  }

  SeniWasm.setString(SeniWasm.traits_buffer, traits);
  SeniWasm.nextGenerationBuild(genotypes.length, populationSize,
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

function configureWasmModule(wasmInstance) {
  const w = wasmInstance;

  SeniWasm.instance = w;

  // declare string functions
  SeniWasm.setString = w.memory.setString;
  SeniWasm.getString = w.memory.getString;

  // declare Sen's wasm insterface
  SeniWasm.senStartup = w.exports.sen_startup;
  SeniWasm.senShutdown = w.exports.sen_shutdown;
  SeniWasm.scriptCleanup = w.exports.script_cleanup;

  SeniWasm.compileToRenderPackets = w.exports.compile_to_render_packets;
  SeniWasm.getRenderPacketNumVertices = w.exports.get_render_packet_num_vertices;
  SeniWasm.getRenderPacketVBuf = w.exports.get_render_packet_vbuf;
  SeniWasm.getRenderPacketCBuf = w.exports.get_render_packet_cbuf;
  SeniWasm.getRenderPacketTBuf = w.exports.get_render_packet_tbuf;

  SeniWasm.buildTraits = w.exports.build_traits;
  SeniWasm.createInitialGeneration = w.exports.create_initial_generation;
  SeniWasm.singleGenotypeFromSeed = w.exports.single_genotype_from_seed;
  SeniWasm.genotypeMoveToBuffer = w.exports.genotype_move_to_buffer;
  SeniWasm.useGenotypeWhenCompiling = w.exports.use_genotype_when_compiling;
  SeniWasm.unparseWithGenotype = w.exports.unparse_with_genotype;
  SeniWasm.simplifyScript = w.exports.simplify_script;

  SeniWasm.nextGenerationPrepare = w.exports.next_generation_prepare;
  SeniWasm.nextGenerationAddGenotype = w.exports.next_generation_add_genotype;
  SeniWasm.nextGenerationBuild = w.exports.next_generation_build;

  SeniWasm.getSourceBuffer = w.exports.get_source_buffer;
  SeniWasm.getOutSourceBuffer = w.exports.get_out_source_buffer;
  SeniWasm.getTraitsBuffer = w.exports.get_traits_buffer;
  SeniWasm.getGenotypeBuffer = w.exports.get_genotype_buffer;
}

/*
function freeModule() {

  Module._free(SeniWasm.ptr);
  Module._free(SeniWasm.vbuf);
  Module._free(SeniWasm.cbuf);
  Module._free(SeniWasm.tbuf);

  SeniWasm.senShutdown();
}
*/




// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

function messageHandler(type, data) {
  switch (type) {
  case jobRender:
    return render(data);
  case jobUnparse:
    return unparse(data);
  case jobBuildTraits:
    return buildTraits(data);
  case jobInitialGeneration:
    return createInitialGeneration(data);
  case jobSingleGenotypeFromSeed:
    return singleGenotypeFromSeed(data);
  case jobSimplifyScript:
    return simplifyScript(data);
  case jobNewGeneration:
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



function main() {
  console.log('hello from main');
  lenlen();
  say_hi();

/*
  configureWasmModule(wasmInstance);

  SeniWasm.senStartup();
  // get string buffers
  SeniWasm.source_buffer = SeniWasm.getSourceBuffer();
  SeniWasm.out_source_buffer = SeniWasm.getOutSourceBuffer();
  SeniWasm.traits_buffer = SeniWasm.getTraitsBuffer();
  SeniWasm.genotype_buffer = SeniWasm.getGenotypeBuffer();
*/

  // send the job system an initialised message so
  // that it can start sending jobs to this worker
  const sendData = JSON.stringify([{systemInitialised: true}]);
  postMessage(sendData);

}

wasm_bindgen('./sen_client_bg.wasm')
  .then(() => {
    // hack to access the memory
    // the build.sh has a sed command to export the wasm object
    // replace the js renderer with a rust implmentation to get rid of this hack
    // memory = wasm_bindgen.wasm.memory;
    main();
  })
  .catch(console.error);
