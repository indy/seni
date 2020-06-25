let gMuhPrefix = '';

// based on code from:
// https://hackernoon.com/functional-javascript-resolving-promises-sequentially-7aac18c4431e
function sequentialPromises(funcs) {
  return funcs.reduce((promise, func) =>
    promise.then(result => func().then(Array.prototype.concat.bind(result))),
    Promise.resolve([]));
}

// todo: is this the best way of getting image data for a web worker?
// is there a way for the webworker to do this without having to interact with the DOM?
// note: don't call this on a sequence of bitmaps
function loadBitmapImageData(url) {
  return new Promise(function(resolve, reject) {
    const element = document.getElementById('bitmap-canvas');
    const context = element.getContext('2d');
    const img = new Image();

    img.onload = () => {
      element.width = img.width;
      element.height = img.height;

      context.drawImage(img, 0, 0);

      const imageData = context.getImageData(0, 0, element.width, element.height);

      resolve(imageData);
    };
    img.onerror = () => {
      reject();
    };

    img.src = normalize_bitmap_url(url);
  });
}

function normalize_bitmap_url(url) {
  const re = /^[\w-/]+.png/;

  if (url.match(re)) {
    // requesting a bitmap just by filename, so get it from /img/immutable/
    return gMuhPrefix + "/img/immutable/" + url;
  } else {
    // change nothing, try and fetch the url
    return url;
  }
}

function sleepy(timeout) {
  return new Promise((resolve, reject) => {
    setTimeout(() => {
      resolve();
    }, timeout);
  });
}

async function renderJob(parameters) {
  // 1. compile the program in a web worker
  // 2. (retain the id for this worker)
  // 3. after compilation, the worker will return a list of bitmaps that are
  //    required by the program and are not in the web worker's bitmap-cache
  // 4. sequentially load in the bitmaps and send their data to the worker
  // 5. can now request a render which will return the render packets

  // request a compile job but make sure to retain the worker as it will be performing the rendering
  //
  parameters.__retain = true;
  const { bitmapsToTransfer, __worker_id } = await Job.request(JobType.jobRender_1_Compile, parameters);

  // convert each bitmap path to a function that returns a promise
  //
  const bitmap_loading_funcs = bitmapsToTransfer.map(filename => async () => {
    Log.log(`worker ${__worker_id}: bitmap request: ${filename}`);

    const imageData = await loadBitmapImageData(filename);
    // make an explicit job request to the same worker
    return Job.request(JobType.jobRender_2_ReceiveBitmapData, { filename, imageData, __retain: true }, __worker_id);
  });

  // seqentially execute the promises that load in bitmaps and send the bitmap data to a particular worker
  //
  await sequentialPromises(bitmap_loading_funcs);

  // now make an explicit job request to the same worker that has recieved the bitmap data
  // note: no __retain as we want the worker to be returned to the available pool
  const renderPacketsResult = await Job.request(JobType.jobRender_3_RenderPackets, {}, __worker_id);

  return renderPacketsResult;
}

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

async function loadShaders(scriptUrls) {
  const fetchPromises = scriptUrls.map(s => fetch(s));
  const responses = await Promise.all(fetchPromises);

  const textPromises = responses.map(r => r.text());
  const shaders = await Promise.all(textPromises);

  const res = {};
  for (const [i, url] of scriptUrls.entries()) {
    res[url] = shaders[i];
  }

  return res;
}

async function getJSON(url) {
  const res = await fetch(url);
  const json = await res.json();
  return json;
}


function getRequiredElement(id) {
  const element = document.getElementById(id);
  if (!element) {
    console.error(`required element ${id} not found in dom`);
  }
  return element;
}

function addClass(id, clss) {
  const e = getRequiredElement(id);
  e.classList.add(clss);
}

function removeClass(id, clss) {
  const e = getRequiredElement(id);
  e.classList.remove(clss);
}

function removeAllChildren(e) {
  while (e.firstChild) {
    e.removeChild(e.firstChild);
  }
}

function setOpacity(id, opacity) {
  const e = getRequiredElement(id);
  e.style.opacity = opacity;
}

function getURIParameters() {
  const argPairs = window.location.search.substring(1).split("&");

  return argPairs.reduce((acc, kv) => {
    let [key, value] = kv.split("=");
    if (key === URI_SEED) {
      acc[key] = parseInt(value, 10);
    } else {
      acc[key] = value;
    }

    return acc;
  }, {});
}

function addClickEvent(id, fn) {
  const element = document.getElementById(id);

  if (element) {
    element.addEventListener('click', fn);
  } else {
    console.error('cannot addClickEvent for', id);
  }
}

function countDigits(num) {
  if(num < 10) {
    return 1;
  } else if (num < 100) {
    return 2;
  } else if (num < 1000) {
    return 3;
  } else if (num < 10000) {
    return 4;
  } else {
    console.error(`countDigits given an insanely large number: ${num}`);
    return 5;
  }
}

// https://developer.mozilla.org/en-US/docs/Web/Events/resize
function throttle(type, name, obj) {
  const obj2 = obj || window;
  let running = false;
  const func = () => {
    if (running) { return; }
    running = true;
    requestAnimationFrame(() => {
      obj2.dispatchEvent(new CustomEvent(name));
      running = false;
    });
  };
  obj2.addEventListener(type, func);
}
