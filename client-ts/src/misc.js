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
