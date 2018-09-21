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

import { jobRender,
         jobUnparse,
         jobBuildTraits,
         jobInitialGeneration,
         jobNewGeneration,
         jobSingleGenotypeFromSeed,
         jobSimplifyScript
       } from './jobTypes';
const SenWasm = {};

const getOwnPropertyNames = Object.getOwnPropertyNames;
/* eslint-disable no-param-reassign */
/* eslint-disable no-multi-spaces */
/* eslint-disable no-return-assign */
function loadWASM(file, options) {
  if (!options) {
    options = {};
  }

  const imports = options.imports || {};

//  imports.__assert_fail = function() {};
//  imports.__floatscan = function() {};
//  imports.__shlim = function() {};

  // Initialize memory

  let memory = imports.memory;
  if (!memory) {
    const opts = { initial: options.initialMemory || 1 };
    if (options.maximumMemory) {
      opts.maximum = options.maximumMemory;
    }
    memory = new WebAssembly.Memory(opts);
    memory.initial = options.initialMemory || 1;
    memory.maximum = options.maximumMemory;
  }

  let table = imports.table;
  if (!table) {
    table = new WebAssembly.Table({ initial: 0, element: 'anyfunc' });
  }


  function grow() {
    const buf = memory.buffer;
    memory.U8 = new Uint8Array(buf);
    memory.S32 = new Int32Array(buf);
    memory.U32 = new Uint32Array (buf);
    memory.F32 = new Float32Array(buf);
    memory.F64 = new Float64Array(buf);
  }

  grow();

  // Add utilty to memory

  /**
   * Reads a 32-bit signed integer starting at the specified memory offset.
   * @typedef GetInt
   * @function
   * @param {number} ptr Memory offset
   * @returns {number} Signed 32-bit integer value
   */
  function getInt(ptr) {
    return memory.S32[ptr >> 2];
  }

  memory.getInt = getInt;

  /**
   * Reads a 32-bit unsigned integer starting at the specified memory offset.
   * @typedef GetUint
   * @function
   * @param {number} ptr Memory offset
   * @returns {number} Unsigned 32-bit integer value
   */
  function getUint(ptr) {
    return memory.U32[ptr >> 2];
  }

  memory.getUint = getUint;

  /**
   * Reads a 32-bit float starting at the specified memory offset.
   * @typedef GetFloat
   * @function
   * @param {number} ptr Memory offset
   * @returns {number} 32-bit float value
   */
  function getFloat(ptr) {
    return memory.F32[ptr >> 2];
  }

  memory.getFloat = getFloat;

  /**
   * Reads a 64-bit double starting at the specified memory offset.
   * @typedef GetDouble
   * @function
   * @param {number} ptr Memory offset
   * @returns {number} 64-bit float value
   */
  function getDouble(ptr) {
    return memory.F64[ptr >> 3];
  }

  memory.getDouble = getDouble;

  /**
   * Reads a (zero-terminated, exclusive) string at the specified memory offset.
   * @typedef GetString
   * @function
   * @param {number} ptr Memory offset
   * @returns {string} String value
   */
  function getString(ptr) {
    const start = (ptr >>>= 0);
    while (memory.U8[ptr++]);
    getString.bytes = ptr - start;
    return String.fromCharCode.apply(null, memory.U8.subarray(start, ptr - 1));
  }

  memory.getString = getString;


  function setString(ptr, str) {
    ptr >>>= 0;

    const strLen = str.length;
    for (let i = 0; i < strLen; i++) {
      memory.U8[ptr++] = str.charCodeAt(i);
    }
    memory.U8[ptr++] = 0;
  }

  memory.setString = setString;

  // Initialize environment

  const env = {};

  env.memoryBase = imports.memoryBase || 0;
  env.memory = memory;
  env.tableBase = imports.tableBase || 0;
  env.table = table;

  // Add console to environment

  function sprintf(ptr, base) {
    const s = getString(ptr);
    return base
      ? s.replace(/%([dfisu]|lf)/g, ($0, $1) => {
        let val;
        return base +=
          $1 === 'u'  ? (val = getUint(base), 4)
          : $1 === 'f'  ? (val = getFloat(base), 4)
          : $1 === 's'  ? (val = getString(getUint(base)), 4)
          : $1 === 'lf' ? (val = getDouble(base), 8)
          :               (val = getInt(base), 4)
        , val;
      })
    : s;
  }

  getOwnPropertyNames(console).forEach(key => {
    if (typeof console[key] === 'function') {// eslint-disable-line no-console
      env[`console_${key}`] = (ptr, base) => {
        console[key](sprintf(ptr, base)); // eslint-disable-line no-console
      };
    }
  });

  // Add Math to environment

  getOwnPropertyNames(Math).forEach(key => {
    if (typeof Math[key] === 'function') {
      env[`Math_${key}`] = Math[key];
    }
  });

  // Add imports to environment

  Object.keys(imports).forEach(key => env[key] = imports[key]);

  // Add default exit listeners if not explicitly imported

  if (!env._abort) {
    env._abort = errno => {
      throw Error(`abnormal abort in ${file}: ${errno}`);
    };
  }
  if (!env._exit) {
    env._exit = code => {
      if (code) {
        throw Error(`abnormal exit in ${file}: ${code}`);
      }
    };
  }


  // Finally, fetch the assembly and instantiate it

  env._grow = grow;

  return fetch(file)
    .then(result => result.arrayBuffer())
    .then(buffer => WebAssembly.instantiate(buffer, { env }))
    .then(module => {
      const instance = module.instance;
      instance.imports = imports;
      instance.memory = memory;
      instance.env = env;
      return instance;
    });
}
/* eslint-enable no-return-assign */
/* eslint-enable no-multi-spaces */
/* eslint-enable no-param-reassign */

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
  return SenWasm.instance.memory.F32.subarray(pos, pos + length);
}

function pointerToArrayBufferCopy(ptr, length) {
  const nByte = 4;
  const pos = ptr / nByte;
  return SenWasm.instance.memory.F32.slice(pos, pos + length);
}
*/

function render({ script /*, scriptHash*/, genotype }) {
  konsoleProxy.clear();
  const buffers = [];

  if (genotype) {
    // console.log(`renderWasm genotype: ${genotype}`);
    SenWasm.useGenotypeWhenCompiling(true);
    SenWasm.setString(SenWasm.genotype_buffer, genotype);
  } else {
    SenWasm.useGenotypeWhenCompiling(false);
  }

  // need to setString before calling compileToRenderPackets
  SenWasm.setString(SenWasm.source_buffer, script);
  const numRenderPackets = SenWasm.compileToRenderPackets();
  // konsoleProxy.log(`numRenderPackets = ${numRenderPackets}`);

  for (let i = 0; i < numRenderPackets; i++) {
    const numVertices = SenWasm.getRenderPacketNumVertices(i);
    // konsoleProxy.log(`render_packet ${i}: numVertices = ${numVertices}`);

    if (numVertices > 0) {
      const buffer = {};

      buffer.vbufAddress = SenWasm.getRenderPacketVBuf(i);
      buffer.cbufAddress = SenWasm.getRenderPacketCBuf(i);
      buffer.tbufAddress = SenWasm.getRenderPacketTBuf(i);

      buffer.numVertices = numVertices;
      buffers.push(buffer);
    }
  }

  SenWasm.scriptCleanup();

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
  const wasmMemory = SenWasm.instance.memory.buffer;
  const memory = wasmMemory.slice();

  return [{ ok: true, logMessages }, { title, memory, buffers }];
}

function unparse({ script/*, scriptHash*/, genotype }) {
  konsoleProxy.clear();

  // console.log(`genotype is ${genotype}`);
  // console.log(`script is ${script}`);

  SenWasm.setString(SenWasm.source_buffer, script);
  SenWasm.setString(SenWasm.genotype_buffer, genotype);

  SenWasm.unparseWithGenotype();
  const newScript = SenWasm.getString(SenWasm.out_source_buffer);

  // console.log(`new script: ${newScript}`);

  const logMessages = konsoleProxy.collectMessages();

  return [{ ok: true, logMessages }, { script: newScript }];
}

function buildTraits({ script /*, scriptHash */ }) {
  konsoleProxy.clear();

  SenWasm.setString(SenWasm.source_buffer, script);

  konsoleProxy.log('worker:buildTraits');
  const numTraits = SenWasm.buildTraits();
  // konsoleProxy.log(`built ${numTraits} traits`);

  const traits = SenWasm.getString(SenWasm.traits_buffer);

  const logMessages = konsoleProxy.collectMessages();

  return [{ ok: true, logMessages }, { traits }];
}

// transfers the contents of g_genotype_list from the wasm side
function getGenotypesFromWasm(populationSize) {
  const genotypes = [];
  let s;

  for (let i = 0; i < populationSize; i++) {
    SenWasm.genotypeMoveToBuffer(i);
    s = SenWasm.getString(SenWasm.genotype_buffer);
    genotypes.push(s);
  }

  return genotypes;
}

function createInitialGeneration({ populationSize, traits }) {
  konsoleProxy.clear();

  SenWasm.setString(SenWasm.traits_buffer, traits);

  const seed = Math.floor(Math.random() * 1024);
  // konsoleProxy.log(`createInitialGeneration seed: ${seed}`);
  // konsoleProxy.log(`createInitialGeneration populationSize: ${populationSize}`);

  SenWasm.createInitialGeneration(populationSize, seed);

  const genotypes = getGenotypesFromWasm(populationSize);

  const logMessages = konsoleProxy.collectMessages();

  return [{ok: true, logMessages}, { genotypes }];
}

function singleGenotypeFromSeed({ seed, traits }) {
  konsoleProxy.clear();

  SenWasm.setString(SenWasm.traits_buffer, traits);

  // konsoleProxy.log(`singleGenotypeFromSeed seed: ${seed}`);

  SenWasm.singleGenotypeFromSeed(seed);

  const genotypes = getGenotypesFromWasm(1);

  const logMessages = konsoleProxy.collectMessages();

  return [{ok: true, logMessages}, { genotype: genotypes[0] }];
}

function simplifyScript({ script }) {
  konsoleProxy.clear();

  SenWasm.setString(SenWasm.source_buffer, script);

  SenWasm.simplifyScript();

  const newScript = SenWasm.getString(SenWasm.out_source_buffer);

  const logMessages = konsoleProxy.collectMessages();

      return [{ ok: true, logMessages }, { script: newScript }];
}

function newGeneration({genotypes, populationSize, traits, mutationRate, rng}) {
  konsoleProxy.clear();

  SenWasm.nextGenerationPrepare();
  for (let i = 0; i < genotypes.length; i++) {
    SenWasm.setString(SenWasm.genotype_buffer, genotypes[i]);
    SenWasm.nextGenerationAddGenotype();
  }

  SenWasm.setString(SenWasm.traits_buffer, traits);
  SenWasm.nextGenerationBuild(genotypes.length, populationSize,
                             mutationRate, rng);

  const newGenotypes = getGenotypesFromWasm(populationSize);

  const logMessages = konsoleProxy.collectMessages();

  return [{ok: true, logMessages }, { genotypes: newGenotypes }];
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

  SenWasm.instance = w;

  // declare string functions
  SenWasm.setString = w.memory.setString;
  SenWasm.getString = w.memory.getString;

  // declare Sen's wasm insterface
  SenWasm.senStartup = w.exports.sen_startup;
  SenWasm.senShutdown = w.exports.sen_shutdown;
  SenWasm.scriptCleanup = w.exports.script_cleanup;

  SenWasm.compileToRenderPackets = w.exports.compile_to_render_packets;
  SenWasm.getRenderPacketNumVertices = w.exports.get_render_packet_num_vertices;
  SenWasm.getRenderPacketVBuf = w.exports.get_render_packet_vbuf;
  SenWasm.getRenderPacketCBuf = w.exports.get_render_packet_cbuf;
  SenWasm.getRenderPacketTBuf = w.exports.get_render_packet_tbuf;

  SenWasm.buildTraits = w.exports.build_traits;
  SenWasm.createInitialGeneration = w.exports.create_initial_generation;
  SenWasm.singleGenotypeFromSeed = w.exports.single_genotype_from_seed;
  SenWasm.genotypeMoveToBuffer = w.exports.genotype_move_to_buffer;
  SenWasm.useGenotypeWhenCompiling = w.exports.use_genotype_when_compiling;
  SenWasm.unparseWithGenotype = w.exports.unparse_with_genotype;
  SenWasm.simplifyScript = w.exports.simplify_script;

  SenWasm.nextGenerationPrepare = w.exports.next_generation_prepare;
  SenWasm.nextGenerationAddGenotype = w.exports.next_generation_add_genotype;
  SenWasm.nextGenerationBuild = w.exports.next_generation_build;

  SenWasm.getSourceBuffer = w.exports.get_source_buffer;
  SenWasm.getOutSourceBuffer = w.exports.get_out_source_buffer;
  SenWasm.getTraitsBuffer = w.exports.get_traits_buffer;
  SenWasm.getGenotypeBuffer = w.exports.get_genotype_buffer;
}

/*
function freeModule() {

  Module._free(SenWasm.ptr);
  Module._free(SenWasm.vbuf);
  Module._free(SenWasm.cbuf);
  Module._free(SenWasm.tbuf);

  SenWasm.senShutdown();
}
*/


// set this to true when building for the indy.io gallery
const loadForWebsite = false;

let wasmFile = loadForWebsite ? '/seni/sen-wasm.wasm' : 'sen-wasm.wasm';

loadWASM(wasmFile, options).then(wasmInstance => {
  configureWasmModule(wasmInstance);
  SenWasm.senStartup();
  // get string buffers
  SenWasm.source_buffer = SenWasm.getSourceBuffer();
  SenWasm.out_source_buffer = SenWasm.getOutSourceBuffer();
  SenWasm.traits_buffer = SenWasm.getTraitsBuffer();
  SenWasm.genotype_buffer = SenWasm.getGenotypeBuffer();

  // send the job system an initialised message so
  // that it can start sending jobs to this worker
  const sendData = JSON.stringify([{systemInitialised: true}]);
  postMessage(sendData);
});

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
 ok: true
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
      postMessage(sendData);
    }
  } catch (error) {
    postMessage(JSON.stringify([{error: {message: error.message}}, undefined]));
  }
});
