/*
 *  Copyright (C) 2020 Inderjit Gill <email@indy.io>
 *
 *  This file is part of Seni
 *
 *  Seni is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Seni is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

importScripts('client.js');

// global state
let seniBridge = undefined;
let seniMemory = undefined;

// javascript output from typescript compilation of JobType.ts
let JobType;
(function (JobType) {
    JobType[JobType["jobRender_1_Compile"] = 0] = "jobRender_1_Compile";
    JobType[JobType["jobRender_2_ReceiveBitmapData"] = 1] = "jobRender_2_ReceiveBitmapData";
    JobType[JobType["jobRender_3_RenderPackets"] = 2] = "jobRender_3_RenderPackets";
    JobType[JobType["jobUnparse"] = 3] = "jobUnparse";
    JobType[JobType["jobBuildTraits"] = 4] = "jobBuildTraits";
    JobType[JobType["jobInitialGeneration"] = 5] = "jobInitialGeneration";
    JobType[JobType["jobNewGeneration"] = 6] = "jobNewGeneration";
    JobType[JobType["jobGenerateHelp"] = 7] = "jobGenerateHelp";
    JobType[JobType["jobSingleGenotypeFromSeed"] = 8] = "jobSingleGenotypeFromSeed";
    JobType[JobType["jobSimplifyScript"] = 9] = "jobSimplifyScript";
})(JobType || (JobType = {}));

function compile({ script, genotype }) {
  if (genotype) {
    // console.log("render: using a genotype");
    seniBridge.compile_program_from_source_and_genotype(script, genotype);

  } else {
    seniBridge.compile_program_from_source(script);
  }

  const bitmapsToTransfer = JSON.parse(seniBridge.get_bitmap_transfers_as_json());

  return [{}, { bitmapsToTransfer }];
}

function receiveBitmapData( { filename, imageData } ) {
  // todo: see if the imageData.data can be transferred across
  const pixels = [];
  const numElements = imageData.width * imageData.height * 4;
  for (i = 0; i < numElements; i++) {
    pixels.push(imageData.data[i]);
  }

  seniBridge.add_rgba_bitmap(filename, imageData.width, imageData.height, pixels);

  return [{}, { result: "shabba" }];
}

const RPCommand_Geometry = 1;
const RPCommand_Mask = 2;
const RPCommand_Image = 3;

function renderPackets({  }) {
  const buffers = [];

  const numRenderPackets = seniBridge.run_program();

  for (let i = 0; i < numRenderPackets; i++) {
    const buffer = {};
    buffer.command = seniBridge.rp_command(i);

    switch (buffer.command) {
    case RPCommand_Geometry:
      const renderPacketGeometry = seniBridge.rp_geometry(i);

      buffer.geo_len = renderPacketGeometry.get_geo_len();
      buffer.geo_ptr = renderPacketGeometry.get_geo_ptr();

      renderPacketGeometry.free();
      break;
    case RPCommand_Mask:
      const renderPacketMask = seniBridge.rp_mask(i);

      buffer.mask_filename = renderPacketMask.get_filename();
      buffer.mask_invert = renderPacketMask.get_invert();

      renderPacketMask.free();
      break;
    case RPCommand_Image:
      const renderPacketImage = seniBridge.rp_image(i);

      buffer.linearColourSpace = renderPacketImage.get_linear_colour_space();
      buffer.contrast = renderPacketImage.get_contrast();
      buffer.brightness = renderPacketImage.get_brightness();
      buffer.saturation = renderPacketImage.get_saturation();

      renderPacketImage.free();
      break;
    default:
      console.error(`unknown buffer command: ${buffer.command}`);
      break;
    };

    buffers.push(buffer);
  };

  const meta = {
    title: ''
  };

  seniBridge.script_cleanup();

  // make a copy of the wasm memory
  //
  // note: (05-12-2017) required by Firefox as that doesn't allow transferring
  // Wasm ArrayBuffers to different threads
  // (errors with: cannot transfer WebAssembly/asm.js ArrayBuffer)
  //
  // WTF note: Expected a perfomance cost in Chrome due to the slice operation
  // but it seemed to either have no effect or to make the rendering faster!
  //
  const wasmMemory = seniMemory.buffer;
  const memory = wasmMemory.slice();

  return [{}, { meta, memory, buffers }];
}

function unparse({ script, genotype }) {
  const newScript = seniBridge.unparse_with_genotype(script, genotype);

  return [{}, { script: newScript }];
}

function buildTraits({ script }) {
  const traits = seniBridge.build_traits(script);
  const validTraits = traits !== "";

  return [{}, { validTraits, traits }];
}

// transfers the contents of g_genotype_list from the wasm side
function getGenotypesFromWasm(populationSize) {
  const genotypes = [];
  let s;

  for (let i = 0; i < populationSize; i++) {
    s = seniBridge.get_genotype(i);
    if (s === "") {
      console.error(`getGenotypesFromWasm: error getting genotype: ${i}`);
    }
    genotypes.push(s);
  }

  return genotypes;
}

function createInitialGeneration({ populationSize, traits }) {
  const seed = Math.floor(Math.random() * 1024);

  seniBridge.create_initial_generation(traits, populationSize, seed);

  const genotypes = getGenotypesFromWasm(populationSize);

  return [{}, { genotypes }];
}

function singleGenotypeFromSeed({ seed, traits }) {
  seniBridge.single_genotype_from_seed(traits, seed);

  const genotypes = getGenotypesFromWasm(1);

  return [{}, { genotype: genotypes[0] }];
}

function simplifyScript({ script }) {
  const newScript = seniBridge.simplify_script(script);

  return [{}, { script: newScript }];
}

function newGeneration({genotypes, populationSize, traits, mutationRate, rng}) {
  seniBridge.next_generation_prepare();
  for (let i = 0; i < genotypes.length; i++) {
    seniBridge.next_generation_add_genotype(genotypes[i]);
  }

  seniBridge.next_generation_build(traits,
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

  Module._free(seniBridge.ptr);
  Module._free(seniBridge.vbuf);
  Module._free(seniBridge.cbuf);
  Module._free(seniBridge.tbuf);

  seniBridge.senShutdown();
  }
*/

function messageHandler(type, data) {
  switch (type) {
  case JobType["jobRender_1_Compile"]:
    return compile(data);
  case JobType["jobRender_2_ReceiveBitmapData"]:
    return receiveBitmapData(data);
  case JobType["jobRender_3_RenderPackets"]:
    return renderPackets(data);
  case JobType["jobUnparse"]:
    return unparse(data);
  case JobType["jobBuildTraits"]:
    return buildTraits(data);
  case JobType["jobInitialGeneration"]:
    return createInitialGeneration(data);
  case JobType["jobSingleGenotypeFromSeed"]:
    return singleGenotypeFromSeed(data);
  case JobType["jobSimplifyScript"]:
    return simplifyScript(data);
  case JobType["jobNewGeneration"]:
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

    if (type === JobType["jobRender_3_RenderPackets"]) {
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

    seniBridge = new Bridge();
    seniMemory = client_bg.memory;

    // send the job system an initialised message so
    // that it can start sending jobs to this worker
    postMessage([{systemInitialised: true}, undefined]);

  })
  .catch(console.error);
