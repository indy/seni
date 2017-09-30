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

/* eslint-disable no-unused-vars */

import CodeMirrorSeni from './CodeMirrorSeni';

/* eslint-enable no-unused-vars */

const modeName = 'seni';

function defineSeniMode() {
  // return an instance of CodeMirror with Seni mode defined
  CodeMirror.defineMode(modeName, CodeMirrorSeni.seniMode);
  return CodeMirror;
}

function createEditor(element, customConfig) {
  const codeMirrorSeniMode = defineSeniMode();
  const defaultConfig = {
    lineNumbers: false,
    mode: modeName,
    autoCloseBrackets: true,
    matchBrackets: true
  };
  const res = Object.assign({}, defaultConfig, customConfig);

  return codeMirrorSeniMode.fromTextArea(element, res);
}

export default {
  createEditor
};