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


// THIS IS A PLACEHOLDER FILE

const { say_hi, lenlen } = wasm_bindgen;

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

function main() {
  console.log('hello from main');
  lenlen();
  say_hi();
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();

  wasm_bindgen('./sen_client_bg.wasm')
    .then(() => {
      // hack to access the memory
      // the build.sh has a sed command to export the wasm object
      // replace the js renderer with a rust implmentation to get rid of this hack
      // memory = wasm_bindgen.wasm.memory;
      main();
    })
    .catch(console.error);
});
