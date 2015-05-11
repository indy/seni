
/* eslint-disable no-unused-vars */

import CodeMirror from '../../node_modules/codemirror/lib/codemirror';
import * as cb from '../../node_modules/codemirror/addon/edit/closebrackets';
import * as mb from '../../node_modules/codemirror/addon/edit/matchbrackets';
import CodeMirrorSeni from './CodeMirrorSeni';

const modeName = 'seni';

const defaultConfig = {
    lineNumbers: false,
    mode: modeName,
    autoCloseBrackets: true,
    matchBrackets: true
};

const CodeMirrorConfig = {
  getCodeMirror() {
    // return an instance of CodeMirror with Seni mode defined
    CodeMirror.defineMode(modeName, CodeMirrorSeni.seniMode);
    return CodeMirror;
  },
  defaultConfig,
  modeName
};

export default CodeMirrorConfig;
