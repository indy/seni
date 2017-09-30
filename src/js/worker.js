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

import { jobRenderWasm,
         jobUnparse,
         jobBuildTraits,
         jobInitialGeneration,
         jobNewGeneration
       } from './jobTypes';
const Shabba = {};

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
  return Shabba.instance.memory.F32.subarray(pos, pos + length);
}

function pointerToArrayBufferCopy(ptr, length) {
  const nByte = 4;
  const pos = ptr / nByte;
  return Shabba.instance.memory.F32.slice(pos, pos + length);
}
*/

function renderWasm({ script /*, scriptHash*/, genotype }) {
  konsoleProxy.clear();

  const buffers = [];

  if (genotype) {
    // console.log(`renderWasm genotype: ${genotype}`);
    Shabba.useGenotypeWhenCompiling(true);
    Shabba.setString(Shabba.genotype_buffer, genotype);
  } else {
    Shabba.useGenotypeWhenCompiling(false);
  }

  // need to setString before calling compileToRenderPackets
  Shabba.setString(Shabba.source_buffer, script);
  const numRenderPackets = Shabba.compileToRenderPackets();
  // console.log(`numRenderPackets = ${numRenderPackets}`);

  for (let i = 0; i < numRenderPackets; i++) {
    const numVertices = Shabba.getRenderPacketNumVertices(i);
    // console.log(`render_packet ${i}: numVertices = ${numVertices}`);

    if (numVertices > 0) {
      const buffer = {};

      buffer.vbufAddress = Shabba.getRenderPacketVBuf(i);
      buffer.cbufAddress = Shabba.getRenderPacketCBuf(i);
      buffer.tbufAddress = Shabba.getRenderPacketTBuf(i);

      buffer.numVertices = numVertices;
      buffers.push(buffer);
    }
  }

  const memory = Shabba.instance.memory.buffer;

  Shabba.scriptCleanup();

  const logMessages = konsoleProxy.collectMessages();
  const title = 'WASM woohoo';
  return { title, memory, buffers, logMessages };
}

function unparse({ script/*, scriptHash*/, genotype }) {
  // console.log(`genotype is ${genotype}`);
  // console.log(`script is ${script}`);
  Shabba.setString(Shabba.source_buffer, script);
  Shabba.setString(Shabba.genotype_buffer, genotype);

  Shabba.unparseWithGenotype();
  const newScript = Shabba.getString(Shabba.out_source_buffer);

  // console.log(`new script: ${newScript}`);

  return { script: newScript };
}

function buildTraitsWasm({ script /*, scriptHash */ }) {
  Shabba.setString(Shabba.source_buffer, script);

  const numTraits = Shabba.buildTraits();
  console.log(`built ${numTraits} traits`);

  const traits = Shabba.getString(Shabba.traits_buffer);
  // console.log(`js side recieved: ${traits}`);

  return { traits };
}

// transfers the contents of g_genotype_list from the wasm side
function getGenotypesFromWasm(populationSize) {
  const genotypes = [];
  let s;

  for (let i = 0; i < populationSize; i++) {
    Shabba.genotypeMoveToBuffer(i);
    s = Shabba.getString(Shabba.genotype_buffer);
    genotypes.push(s);
  }

  return genotypes;
}

function createInitialGenerationWasm({ populationSize, traits }) {
  console.log('createInitialGenerationWasm');
  Shabba.setString(Shabba.traits_buffer, traits);

  const seed = Math.floor(Math.random() * 1024);
  console.log(`createInitialGenerationWasm seed: ${seed}`);
  console.log(`createInitialGenerationWasm populationSize: ${populationSize}`);

  Shabba.createInitialGeneration(populationSize, seed);

  const genotypes = getGenotypesFromWasm(populationSize);

  return { genotypes };
}

function newGeneration({genotypes, populationSize, traits, mutationRate, rng}) {
  Shabba.nextGenerationPrepare();
  for (let i = 0; i < genotypes.length; i++) {
    Shabba.setString(Shabba.genotype_buffer, genotypes[i]);
    Shabba.nextGenerationAddGenotype();
  }

  Shabba.setString(Shabba.traits_buffer, traits);
  Shabba.nextGenerationBuild(genotypes.length, populationSize,
                             mutationRate, rng);

  const newGenotypes = getGenotypesFromWasm(populationSize);

  return { genotypes: newGenotypes };
}

function register(callback) {
  self.addEventListener('message', e => {
    try {
      const { type, data } = JSON.parse(e.data);

      const result = callback(type, data);

      if (type === jobRenderWasm) {
        const transferrable = [];

        if (result.buffers.length > 0) {
          transferrable.push(result.memory);
        }

        self.postMessage([null, result], transferrable);
      } else {
        const sendData = JSON.stringify([null, result]);
        self.postMessage(sendData);
      }
    } catch (error) {
      self.postMessage(JSON.stringify([{message: error.message}]));
    }
  });
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

  Shabba.instance = w;

  // declare string functions
  Shabba.setString = w.memory.setString;
  Shabba.getString = w.memory.getString;

  // declare Seni's wasm insterface
  Shabba.seniStartup = w.exports.seni_startup;
  Shabba.seniShutdown = w.exports.seni_shutdown;
  Shabba.scriptCleanup = w.exports.script_cleanup;

  Shabba.compileToRenderPackets = w.exports.compile_to_render_packets;
  Shabba.getRenderPacketNumVertices = w.exports.get_render_packet_num_vertices;
  Shabba.getRenderPacketVBuf = w.exports.get_render_packet_vbuf;
  Shabba.getRenderPacketCBuf = w.exports.get_render_packet_cbuf;
  Shabba.getRenderPacketTBuf = w.exports.get_render_packet_tbuf;

  Shabba.buildTraits = w.exports.build_traits;
  Shabba.createInitialGeneration = w.exports.create_initial_generation;
  Shabba.genotypeMoveToBuffer = w.exports.genotype_move_to_buffer;
  Shabba.useGenotypeWhenCompiling = w.exports.use_genotype_when_compiling;
  Shabba.unparseWithGenotype = w.exports.unparse_with_genotype;

  Shabba.nextGenerationPrepare = w.exports.next_generation_prepare;
  Shabba.nextGenerationAddGenotype = w.exports.next_generation_add_genotype;
  Shabba.nextGenerationBuild = w.exports.next_generation_build;

  Shabba.getSourceBuffer = w.exports.get_source_buffer;
  Shabba.getOutSourceBuffer = w.exports.get_out_source_buffer;
  Shabba.getTraitsBuffer = w.exports.get_traits_buffer;
  Shabba.getGenotypeBuffer = w.exports.get_genotype_buffer;
}

/*
function freeModule() {

  Module._free(Shabba.ptr);
  Module._free(Shabba.vbuf);
  Module._free(Shabba.cbuf);
  Module._free(Shabba.tbuf);

  Shabba.seniShutdown();
}
*/

//console.log("about to start");
loadWASM('seni-wasm.wasm', options).then(wasmInstance => {
  configureWasmModule(wasmInstance);
  Shabba.seniStartup();
  // get string buffers
  Shabba.source_buffer = Shabba.getSourceBuffer();
  Shabba.out_source_buffer = Shabba.getOutSourceBuffer();
  Shabba.traits_buffer = Shabba.getTraitsBuffer();
  Shabba.genotype_buffer = Shabba.getGenotypeBuffer();

  register((type, data) => {
    switch (type) {
    case jobRenderWasm:
      return renderWasm(data);
    case jobUnparse:
      return unparse(data);
    case jobBuildTraits:
      return buildTraitsWasm(data);
    case jobInitialGeneration:
      return createInitialGenerationWasm(data);
    case jobNewGeneration:
      return newGeneration(data);
    default:
      // throw unknown type
      throw new Error(`worker.js: Unknown type: ${type}`);
    }
  });
});