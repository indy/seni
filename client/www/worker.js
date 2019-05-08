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

importScripts('client.js');

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
const jobReceiveBitmapData = 'RECEIVE_BITMAP_DATA';

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
  // but it seemed to either have no effect or to make the rendering faster!
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

function receiveBitmapData( { filename, imageData } ) {
  konsoleProxy.clear();

  // todo: see if the imageData.data can be transferred across
  const pixels = [];
  const numElements = imageData.width * imageData.height * 4;
  for (i = 0; i < numElements; i++) {
    pixels.push(imageData.data[i]);
  }

  gState.bridge.add_rgba_bitmap(filename, imageData.width, imageData.height, pixels);

  const logMessages = konsoleProxy.collectMessages();

  return [{ logMessages }, { result: "shabba" }];
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
  case jobReceiveBitmapData:
    // console.log("jobReceiveBitmapData");
    return receiveBitmapData(data);

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

addEventListener('message', event => {
  try {
    const {type, data} = event.data;

    const [status, result] = messageHandler(type, data);

    if (type === jobRender) {
      const transferrable = [];

      if (result.buffers && result.buffers.length > 0) {
        transferrable.push(result.memory);
      }

      // possible bug?: if result.memory is transferred over to the main thread,
      // does that mean that future uses of memory by the worker are unsafe?
      // Is it currently working just through sheer luck?
      //
      postMessage([status, result], transferrable);
    } else {
      postMessage([status, result]);
    }
  } catch (error) {
    postMessage([{error: {message: error.message}}, undefined]);
  }
});

wasm_bindgen('./client_bg.wasm')
  .then(client_bg => {
    const { Bridge, init_client_system } = wasm_bindgen;

    init_client_system();

    gState.bridge = new Bridge();
    gState.memory = client_bg.memory;

    // send the job system an initialised message so
    // that it can start sending jobs to this worker
    postMessage([{systemInitialised: true}, undefined]);

  })
  .catch(console.error);
