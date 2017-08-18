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

import Immutable from 'immutable';

import SpecialDebug from './seni/SpecialDebug';

import Bind from './lang/Bind';
import Runtime from './lang/Runtime';
import Renderer from './seni/Renderer';
import Genetic from './lang/Genetic';
import { jobRender,
         jobRenderWasm,
         jobUnparse,
         jobBuildTraits,
         jobInitialGeneration,
         jobNewGeneration,
         jobGenerateHelp } from './jobTypes';

const logToConsole = false;
const gRenderer = new Renderer();
const gEnv = Bind.addBindings(Runtime.createEnv(), gRenderer);

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
SpecialDebug.useKonsole(konsoleProxy);


let gScriptHash = '';
let gFrontAst = undefined;
let gBackAst = undefined;
let gGenotype = undefined;

// todo: return more informative errors
function updateState(script, scriptHash, genotype) {
  if (scriptHash !== gScriptHash) {
    gScriptHash = scriptHash;

    gFrontAst = Runtime.buildFrontAst(script);
    if (gFrontAst.error) {
      return false;
    }

    gBackAst = Runtime.compileBackAst(gFrontAst.nodes);
  }

  if (genotype !== undefined) {
    gGenotype = genotype;
  } else {
    const traits = Genetic.buildTraits(gBackAst);
    gGenotype = Genetic.createGenotypeFromInitialValues(traits);
  }
  return true;
}

function titleForScript(env, scriptHash) {
  // default the scriptTitle to scriptHash
  // (but replace with 'title' binding if it's defined in the script)
  let scriptTitle = scriptHash;
  if (env) {
    const titleBinding = env.get('title');
    if (titleBinding) {
      scriptTitle = titleBinding.binding;
    }
  }
  return scriptTitle;
}

function render({ script, scriptHash, genotype }) {
  konsoleProxy.clear();
  gRenderer.preDrawScene();

  updateState(script, scriptHash, Immutable.fromJS(genotype));

  const res = Runtime.evalAst(gEnv, gBackAst, gGenotype);
  if (res === undefined) {
    throw new Error('worker.js::render evalAst returned undefined');
  }

  const finalEnv = res[0];
  const title = titleForScript(finalEnv, scriptHash);

  gRenderer.postDrawScene();

  const renderPackets = gRenderer.getRenderPackets();

  const buffers = renderPackets.map(packet => {
    const bufferData = {
      vbuf: packet.abVertex,
      cbuf: packet.abColour,
      tbuf: packet.abTexture,
      numVertices: packet.bufferLevel
    };
    return bufferData;
  });

  const logMessages = konsoleProxy.collectMessages();

  return { title, buffers, logMessages };
}

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

function renderWasm({ script /*, scriptHash, genotype*/ }) {
  konsoleProxy.clear();

  const buffers = [];

  // need to setString before calling compileToRenderPackets
  Shabba.setString(Shabba.source_buffer, script);
  const numRenderPackets = Shabba.compileToRenderPackets();
  console.log(`numRenderPackets = ${numRenderPackets}`);

  for (let i = 0; i < numRenderPackets; i++) {
    const numVertices = Shabba.getRenderPacketNumVertices(i);
    console.log(`render_packet ${i}: numVertices = ${numVertices}`);

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

function unparse({ script, scriptHash, genotype }) {
  updateState(script, scriptHash, Immutable.fromJS(genotype));

  const newScript = Runtime.unparse(gFrontAst.nodes, gGenotype);

  return { script: newScript };
}

/*
// this isn't saving the intermediate ASTs, perhaps do so later?
function buildTraits({ script, scriptHash }) {
  if (scriptHash !== gScriptHash) {
    gScriptHash = scriptHash;
    gFrontAst = Runtime.buildFrontAst(script);

    if (gFrontAst.error) {
      // don't cache the current compilation state variables
      gScriptHash = undefined;
      throw new Error(`worker.js::buildTraits: ${gFrontAst.error}`);
    }
    gBackAst = Runtime.compileBackAst(gFrontAst.nodes);
  }

  const traits = Genetic.buildTraits(gBackAst);

  return { traits };
}
*/

function buildTraitsWasm({ script /*, scriptHash */ }) {
  Shabba.setString(Shabba.source_buffer, script);

  const numTraits = Shabba.buildTraits();
  console.log(`built ${numTraits} traits`);

  const traits = Shabba.getString(Shabba.traits_buffer);
  console.log(`js side recieved: ${traits}`);

  return { traits };
}

/*
function createInitialGeneration({ populationSize, traits }) {
  const random = performance.now();
  const genotypes = [];

  const initialGeno = Genetic.createGenotypeFromInitialValues(traits);
  genotypes.push(initialGeno.toJS());

  for (let i = 1; i < populationSize; i++) {
    const genotype = Genetic.createGenotypeFromTraits(traits, i + random);
    genotypes.push(genotype.toJS());
  }

  return { genotypes };
}
*/

function createInitialGenerationWasm({ populationSize, traits }) {
  console.log('createInitialGenerationWasm');
  Shabba.setString(Shabba.traits_buffer, traits);

  console.log(populationSize);
  Shabba.createFoo(populationSize);
  // Shabba.createInitialGeneration(populationSize);

  const genotypes = [];
  // let s;

  // for (let i = 0; i < populationSize; i++) {
  //   Shabba.genotypeMoveToBuffer(i);
  //   s = Shabba.getString(Shabba.genotype_buffer);
  //   genotypes.push(s);
  // }

  return { genotypes };
}

function newGeneration({genotypes, populationSize, traits, mutationRate, rng}) {
  const newGenotypes = Genetic.nextGeneration(Immutable.fromJS(genotypes),
                                              populationSize,
                                              mutationRate,
                                              traits,
                                              rng);
  return { genotypes: newGenotypes };
}

function generateHelp() {
  // create a hash of document objects
  const res = gEnv.reduce((a, v, k) => {
    a[k] = v.pb.doc;
    return a;
  }, {});

  return res;
}

function register(callback) {
  self.addEventListener('message', e => {
    try {
      const { type, data } = JSON.parse(e.data);

      const result = callback(type, data);

      if (type === jobRender) {
        const transferrable = [];
        result.buffers.forEach(buffer => {
          transferrable.push(buffer.vbuf);
          transferrable.push(buffer.cbuf);
          transferrable.push(buffer.tbuf);
        });
        if (logToConsole) {
          const n = transferrable.length / 3;
          console.log(`sending over ${n} transferrable sets`);
        }
        self.postMessage([null, result], transferrable);
      } else if (type === jobRenderWasm) {
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
  Shabba.createFoo = w.exports.create_foo;
  Shabba.createInitialGeneration = w.exports.create_initial_generation;
  Shabba.genotypeMoveToBuffer = w.exports.genotype_move_to_buffer;

  Shabba.getSourceBuffer = w.exports.get_source_buffer;
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
  Shabba.traits_buffer = Shabba.getTraitsBuffer();
  Shabba.genotype_buffer = Shabba.getGenotypeBuffer();

  register((type, data) => {
    switch (type) {
    case jobRender:
      return render(data);
    case jobRenderWasm:
      return renderWasm(data);
    case jobUnparse:
      return unparse(data);
    case jobBuildTraits:
      return buildTraitsWasm(data);
    case jobInitialGeneration:
      // return createInitialGeneration(data);
      return createInitialGenerationWasm(data);
    case jobNewGeneration:
      return newGeneration(data);
    case jobGenerateHelp:
      return generateHelp(data);
    default:
      // throw unknown type
      throw new Error(`worker.js: Unknown type: ${type}`);
    }
  });
});
