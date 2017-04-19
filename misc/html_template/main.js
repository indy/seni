var Module = {};
var Shabba = {};

fetch('seni-wasm.wasm')
  .then(response => response.arrayBuffer())
  .then(bytes => {
    Module.wasmBinary = bytes;

    var script = document.createElement('script');
    document.body.appendChild(script);

    script.onload = () => {
      setTimeout(configureModule);
    };
    script.src = "seni-wasm.js";
  });

function configureModule()
{
  if (!Module.asm._malloc) {
    // make sure the module code has been loaded
    setTimeout(configureModule);
    return;
  }

  Shabba.buffer_fill = Module.cwrap('buffer_fill', 'number', ['number', 'number', 'string']);

  Shabba.floatSize = 4; // size in bytes of an f32
  Shabba.arrayLength = 4;
  Shabba.ptr = Module._malloc(Shabba.arrayLength * Shabba.floatSize);
}

function freeModule()
{
  Module._free(Shabba.ptr);
}

function pointerToFloat32Array(ptr, length)
{
  var nByte = 4;
  var pos = ptr / nByte;
  return Module.HEAPF32.subarray(pos, pos + length);
}

document.querySelector('.mybutton').addEventListener('click', function(){


  var newLength = Shabba.buffer_fill(Shabba.ptr, Shabba.arrayLength, "howo doodo");
  var moddedArray = pointerToFloat32Array(Shabba.ptr, newLength);

  console.log(Shabba.ptr);
  console.log(newLength);
  console.log(moddedArray);

  console.log("round 2");

  newLength = Shabba.buffer_fill(Shabba.ptr, Shabba.arrayLength, "hello to c-land from main.js");
  moddedArray = pointerToFloat32Array(Shabba.ptr, newLength);

  console.log(Shabba.ptr);
  console.log(newLength);
  console.log(moddedArray);

  freeModule();
});


/*
  // http://stackoverflow.com/questions/41875728/pass-a-javascript-array-as-argument-to-a-webassembly-function

  // Takes an Int32Array, copies it to the heap and returns a pointer
  function arrayToPtr(array) {
    var ptr = Module._malloc(array.length * nByte);
    Module.HEAP32.set(array, ptr / nByte);
    return ptr;
  }

  // Takes a pointer and  array length, and returns a Int32Array from the heap
  function ptrToArray(ptr, length) {
    var array = new Int32Array(length);
    var pos = ptr / nByte;
    array.set(Module.HEAP32.subarray(pos, pos + length));
    return array;
  }
*/
