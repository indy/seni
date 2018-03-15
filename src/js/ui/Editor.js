/*
 *  Senie
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

import CodeMirrorSenie from './CodeMirrorSenie';

/* eslint-enable no-unused-vars */

const modeName = 'senie';

function defineSenieMode() {
  // return an instance of CodeMirror with Senie mode defined
  CodeMirror.defineMode(modeName, CodeMirrorSenie.senieMode);
  return CodeMirror;
}

function createEditor(element, customConfig) {
  const codeMirrorSenieMode = defineSenieMode();
  const defaultConfig = {
    lineNumbers: false,
    mode: modeName,
    autoCloseBrackets: true,
    matchBrackets: true
  };
  const res = Object.assign({}, defaultConfig, customConfig);

  return codeMirrorSenieMode.fromTextArea(element, res);
}

export default {
  createEditor
};
