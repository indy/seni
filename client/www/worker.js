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
const jobReceiveBitmapData = 'RECEIVE_BITMAP_DATA';

function compile({ script /*, scriptHash*/, genotype }) {
  if (genotype) {
    // console.log("render: using a genotype");
    gState.bridge.compile_program_from_source_and_genotype(script, genotype);

  } else {
    gState.bridge.compile_program_from_source(script);
  }

  const bitmapsToTransfer = JSON.parse(gState.bridge.get_bitmap_transfers_as_json());

  return [{}, { bitmapsToTransfer }];
}

function receiveBitmapData( { filename, imageData } ) {
  // todo: see if the imageData.data can be transferred across
  const pixels = [];
  const numElements = imageData.width * imageData.height * 4;
  for (i = 0; i < numElements; i++) {
    pixels.push(imageData.data[i]);
  }

  gState.bridge.add_rgba_bitmap(filename, imageData.width, imageData.height, pixels);

  return [{}, { result: "shabba" }];
}

function renderPackets({  }) {
  const buffers = [];

  const numRenderPackets = gState.bridge.run_program();
  // console.log(`numRenderPackets = ${numRenderPackets}`);

  for (let i = 0; i < numRenderPackets; i++) {
    const buffer = {};
    buffer.geo_len = gState.bridge.get_render_packet_geo_len(i);
    if (buffer.geo_len > 0) {
      buffer.geo_ptr = gState.bridge.get_render_packet_geo_ptr(i);
      buffers.push(buffer);
    }
  }

  const meta = {
    title: '',
    output_linear_colour_space: gState.bridge.output_linear_colour_space()
  };

  gState.bridge.script_cleanup();

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

  return [{}, { meta, memory, buffers }];
}


function unparse({ script/*, scriptHash*/, genotype }) {
  const newScript = gState.bridge.unparse_with_genotype(script, genotype);

  return [{}, { script: newScript }];
}

function buildTraits({ script /*, scriptHash */ }) {
  const traits = gState.bridge.build_traits(script);
  const validTraits = traits !== "";

  return [{}, { validTraits, traits }];
}

// transfers the contents of g_genotype_list from the wasm side
function getGenotypesFromWasm(populationSize) {
  const genotypes = [];
  let s;

  for (let i = 0; i < populationSize; i++) {
    s = gState.bridge.get_genotype(i);
    if (s === "") {
      console.error(`getGenotypesFromWasm: error getting genotype: ${i}`);
    }
    genotypes.push(s);
  }

  return genotypes;
}

function createInitialGeneration({ populationSize, traits }) {
  const seed = Math.floor(Math.random() * 1024);

  gState.bridge.create_initial_generation(traits, populationSize, seed);

  const genotypes = getGenotypesFromWasm(populationSize);

  return [{}, { genotypes }];
}

function singleGenotypeFromSeed({ seed, traits }) {
  gState.bridge.single_genotype_from_seed(traits, seed);

  const genotypes = getGenotypesFromWasm(1);

  return [{}, { genotype: genotypes[0] }];
}

function simplifyScript({ script }) {
  const newScript = gState.bridge.simplify_script(script);

  return [{}, { script: newScript }];
}

function newGeneration({genotypes, populationSize, traits, mutationRate, rng}) {
  gState.bridge.next_generation_prepare();
  for (let i = 0; i < genotypes.length; i++) {
    gState.bridge.next_generation_add_genotype(genotypes[i]);
  }

  gState.bridge.next_generation_build(traits,
                                      genotypes.length, populationSize,
                                      mutationRate, rng);

  const newGenotypes = getGenotypesFromWasm(populationSize);

  return [{}, { genotypes: newGenotypes }];
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
  case jobRender_1_Compile:
    // console.log("jobRender_1_Compile");
    return compile(data);
  case jobRender_2_ReceiveBitmapData:
    // console.log("jobRender_2_ReceiveBitmapData");
    return receiveBitmapData(data);
  case jobRender_3_RenderPackets:
    // console.log("jobRender_3_RenderPackets");
    return renderPackets(data);
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
  }
*/

addEventListener('message', event => {
  try {
    const {type, data} = event.data;

    const [status, result] = messageHandler(type, data);

    if (type === jobRender_3_RenderPackets) {
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
