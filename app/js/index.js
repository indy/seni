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

import main from './app.js';

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

var getOwnPropertyNames = Object.getOwnPropertyNames;

function loadWASM(file, options) {

  options || (options = {});

  var imports = options.imports || {};

//  imports.__assert_fail = function() {};
//  imports.__floatscan = function() {};
//  imports.__shlim = function() {};

  // Initialize memory

  var memory = imports.memory;
  if (!memory) {
    var opts = { initial: options.initialMemory || 1 };
    if (options.maximumMemory)
      opts.maximum = options.maximumMemory;
    memory = new WebAssembly.Memory(opts);
    memory.initial = options.initialMemory || 1;
    memory.maximum = options.maximumMemory;
  }

  var table = imports.table;
  if (!table)
    table = new WebAssembly.Table({ initial: 0, element: "anyfunc" });

  function grow() {
    var buf = memory.buffer;
    memory.U8  = new Uint8Array  (buf);
    memory.S32 = new Int32Array  (buf);
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
var getOwnPropertyNames = Object.getOwnPropertyNames;

function loadWASM(file, options) {

  options || (options = {});

  var imports = options.imports || {};

  // Initialize memory

  var memory = imports.memory;
  if (!memory) {
    var opts = { initial: options.initialMemory || 1 };
    if (options.maximumMemory)
      opts.maximum = options.maximumMemory;
    memory = new WebAssembly.Memory(opts);
    memory.initial = options.initialMemory || 1;
    memory.maximum = options.maximumMemory;
  }

  var table = imports.table;
  if (!table)
    table = new WebAssembly.Table({ initial: 0, element: "anyfunc" });

  function grow() {
    var buf = memory.buffer;
    memory.U8  = new Uint8Array  (buf);
    memory.S32 = new Int32Array  (buf);
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
   * Reads a (zero-terminated, exclusive) string starting at the specified memory offset.
   * @typedef GetString
   * @function
   * @param {number} ptr Memory offset
   * @returns {string} String value
   */
  function getString(ptr) {
    var start = (ptr >>>= 0);
    while (memory.U8[ptr++]);
    getString.bytes = ptr - start;
    return String.fromCharCode.apply(null, memory.U8.subarray(start, ptr - 1));
  }

  memory.getString = getString;


  function setString(ptr, str) {
    ptr >>>= 0;

    var str_len = str.length;
    var i;
    for (i = 0; i < str_len; i++) {
      memory.U8[ptr++] = str.charCodeAt(i);
    }
    memory.U8[ptr++] = 0;
  }

  memory.setString = setString;

  // Initialize environment

  var env = {};

  env.memoryBase = imports.memoryBase || 0;
  env.memory = memory;
  env.tableBase = imports.tableBase || 0;
  env.table = table;

  // Add console to environment

  function sprintf(ptr, base) {
    var s = getString(ptr);
    return base
      ? s.replace(/%([dfisu]|lf)/g, ($0, $1) => {
        var val;
        return base +=
          $1 === "u"  ? (val = getUint(base), 4)
          : $1 === "f"  ? (val = getFloat(base), 4)
          : $1 === "s"  ? (val = getString(getUint(base)), 4)
          : $1 === "lf" ? (val = getDouble(base), 8)
          :               (val = getInt(base), 4)
        , val;
      })
    : s;
  }

  getOwnPropertyNames(console).forEach(key => {
    if (typeof console[key] === "function") // eslint-disable-line no-console
      env["console_" + key] = (ptr, base) => {
        console[key](sprintf(ptr, base)); // eslint-disable-line no-console
      };
  });

  // Add Math to environment

  getOwnPropertyNames(Math).forEach(key => {
    if (typeof Math[key] === "function")
      env["Math_" + key] = Math[key];
  });

  // Add imports to environment

  Object.keys(imports).forEach(key => env[key] = imports[key]);

  // Add default exit listeners if not explicitly imported

  if (!env._abort)
    env._abort = errno => { throw Error("abnormal abort in " + file + ": " + errno); };
  if (!env._exit)
    env._exit = code => { if (code) throw Error("abnormal exit in " + file + ": " + code); };

  // Finally, fetch the assembly and instantiate it

  env._grow = grow;

  return fetch(file)
    .then(result => result.arrayBuffer())
    .then(buffer => WebAssembly.instantiate(buffer, { env: env }))
    .then(module => {
      var instance = module.instance;
      instance.imports = imports;
      instance.memory = memory;
      instance.env = env;
      return instance;
    });
}  function getDouble(ptr) {
    return memory.F64[ptr >> 3];
  }

  memory.getDouble = getDouble;

  /**
   * Reads a (zero-terminated, exclusive) string starting at the specified memory offset.
   * @typedef GetString
   * @function
   * @param {number} ptr Memory offset
   * @returns {string} String value
   */
  function getString(ptr) {
    var start = (ptr >>>= 0);
    while (memory.U8[ptr++]);
    getString.bytes = ptr - start;
    return String.fromCharCode.apply(null, memory.U8.subarray(start, ptr - 1));
  }

  memory.getString = getString;


  function setString(ptr, str) {
    ptr >>>= 0;

    var str_len = str.length;
    var i;
    for (i = 0; i < str_len; i++) {
      memory.U8[ptr++] = str.charCodeAt(i);
    }
    memory.U8[ptr++] = 0;
  }

  memory.setString = setString;

  // Initialize environment

  var env = {};

  env.memoryBase = imports.memoryBase || 0;
  env.memory = memory;
  env.tableBase = imports.tableBase || 0;
  env.table = table;

  // Add console to environment

  function sprintf(ptr, base) {
    var s = getString(ptr);
    return base
      ? s.replace(/%([dfisu]|lf)/g, ($0, $1) => {
        var val;
        return base +=
          $1 === "u"  ? (val = getUint(base), 4)
          : $1 === "f"  ? (val = getFloat(base), 4)
          : $1 === "s"  ? (val = getString(getUint(base)), 4)
          : $1 === "lf" ? (val = getDouble(base), 8)
          :               (val = getInt(base), 4)
        , val;
      })
    : s;
  }

  getOwnPropertyNames(console).forEach(key => {
    if (typeof console[key] === "function") // eslint-disable-line no-console
      env["console_" + key] = (ptr, base) => {
        console[key](sprintf(ptr, base)); // eslint-disable-line no-console
      };
  });

  // Add Math to environment

  getOwnPropertyNames(Math).forEach(key => {
    if (typeof Math[key] === "function")
      env["Math_" + key] = Math[key];
  });

  // Add imports to environment

  Object.keys(imports).forEach(key => env[key] = imports[key]);

  // Add default exit listeners if not explicitly imported

  if (!env._abort)
    env._abort = errno => { throw Error("abnormal abort in " + file + ": " + errno); };
  if (!env._exit)
    env._exit = code => { if (code) throw Error("abnormal exit in " + file + ": " + code); };

  // Finally, fetch the assembly and instantiate it

  env._grow = grow;

  return fetch(file)
    .then(result => result.arrayBuffer())
    .then(buffer => WebAssembly.instantiate(buffer, { env: env }))
    .then(module => {
      var instance = module.instance;
      instance.imports = imports;
      instance.memory = memory;
      instance.env = env;
      return instance;
    });
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();

  loadWASM('dist/seni-wasm.wasm').then(instance => {
    Module.instance = instance; // temp
    main();
  });
});
