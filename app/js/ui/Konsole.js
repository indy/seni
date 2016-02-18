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


/*
 * Based on cs_console by Adam Rensel
 *
 * https://github.com/renz45/cs_console
 *
 * licensed under the MIT License:
 *
 * The MIT License (MIT)
 *
 * Copyright (c) 2013 Adam Rensel, Code School LLC
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */

import CodeMirror from 'codemirror';


/* ------------------------------------------------------------ */
/* eslint-disable no-unused-vars */

class CodeMirrorHelpers {
  constructor(cmInstance) {
    this.cmInstance = cmInstance;
  }

  setLine(lineNumber, content) {
    const lineContent = this.cmInstance.doc.getLine(lineNumber);
    return this.cmInstance.doc.replaceRange(content, {
      line: lineNumber,
      ch: 0
    }, {
      line: lineNumber,
      ch: lineContent.length
    });
  }
}

class KeyActions {
  constructor(options) {
    this._defaultCommands = CodeMirror.commands;
    this.options = options;
  }

  setConsole(console) {
    this.console = console;
    return this.helpers = new CodeMirrorHelpers(console);
  }

  setLine(lineNumber, content) {
    return this.helpers.setLine(lineNumber, content);
  }

  goCharLeft() {
    if (this.isCursorAtPrompt()) {
      return this._defaultCommands.goCharLeft(this.console);
    }
    return undefined;
  }

  delCharBefore() {
    if (this.isCursorAtPrompt()) {
      return this._defaultCommands.delCharBefore(this.console);
    }
    return undefined;
  }

  goDocStart() {
    return this.console.scrollIntoView({
      line: 0,
      ch: 0
    });
  }

  goDocEnd() {
    return this.console.scrollIntoView({
      line: this.consoleLineCount(),
      ch: 0
    });
  }

  goLineStart() {
    const cursorPos = this.console.getCursor();
    return this.console.setCursor({
      line: cursorPos.line,
      ch: this.promptLength()
    });
  }

  delGroupBefore() {
    const cursorStartPos = this.console.getCursor();
    this.console.moveH(-1, `group`);
    const futurePos = this.console.getCursor().ch;
    this.console.setCursor(cursorStartPos);
    if (futurePos >= this.promptLength()) {
      return this._defaultCommands.delGroupBefore(this.console);
    }
    return undefined;
  }

  deleteLine() {
    return this.setLine(this.console.getCursor().line, this.options.prompt);
  }

  consoleLineCount() {
    return this.console.lineCount() - 1;
  }

  promptLength() {
    return this.options.prompt.length;
  }

  isCursorAtPrompt() {
    return this.console.getCursor().ch > this.promptLength();
  }
}

class KonsoleHistory {
  constructor(options) {
    this.storage = {};
    this.currentIndex = 0;
    this.historyLabel = `cs-console-history`;
    this.cachedHistory = [];
    this.maxEntries = 25;

    this.options = options;
    if (this.options.historyLabel) {
      this.historyLabel = `cs-${this.options.historyLabel}-console-history`;
    }
    if (this.options.maxEntries) {
      this.maxEntries = options.maxHistoryEntries;
    }
    if (this.localStorageExists()) {
      this.storage = window.localStorage;
      const localHistory = this.getHistory();
      if (localHistory) {
        this.cachedHistory = localHistory;
      }
      this.currentIndex = this.cachedHistory.length - 1;
    }
  }

  localStorageExists() {

    try {
      return !!(window[`localStorage`] !== null && window.localStorage);
    } catch (_error) {
      const e = _error;
      return false;
    }
  }

  push(item) {
    let currentHistory;

    if (!item) {
      return null;
    }
    currentHistory = this.getHistory();
    if (currentHistory[currentHistory.length - 1] === item) {
      return null;
    }
    currentHistory.push(item);
    if (currentHistory.length >= this.maxEntries) {
      const startSlice = currentHistory.length - this.maxEntries;
      currentHistory = currentHistory.slice(startSlice, currentHistory.length);
    }
    this.cachedHistory = currentHistory;
    this.storage[this.historyLabel] = JSON.stringify(currentHistory);
    return this.currentIndex = currentHistory.length - 1;
  }

  getHistory() {
    if (this.storage[this.historyLabel]) {
      return JSON.parse(this.storage[this.historyLabel]);
    } else {
      return [];
    }
  }

  nextHistory() {
    let history;

    if (this.cachedHistory.length > 0) {
      history = this.cachedHistory[this.currentIndex];
    } else {
      history = ``;
    }
    if (this.currentIndex > 0) {
      this.currentIndex--;
    }
    return history;
  }

  previousHistory() {
    if (this.currentIndex < this.cachedHistory.length - 1) {
      this.currentIndex++;
      return this.cachedHistory[this.currentIndex];
    } else {
      return ``;
    }
  }

  clearHistory() {
    this.storage[this.historyLabel] = `[]`;
    return this.cachedHistory = [];
  }

  resetIndex() {
    return this.currentIndex = this.cachedHistory.length - 1;
  }
}

// todo(isg): remove unused code,
//            remove commandValidate???,
//            remove keys
//
export default class Konsole {
  constructor(el, options) {
    this.keyMap = {
      'Alt-Delete': `delGroupAfter`,
      'Alt-Left': `goGroupLeft`,
      'Alt-Right': `goGroupRight`,
      'Cmd-Right': `goLineEnd`,
      'Ctrl-E': `goLineEnd`,
      'Ctrl-Alt-Backspace': `delGroupAfter`,
      'Delete': `delCharAfter`,
      'End': `goLineEnd`,
      'Home': `goLineStartSmart`,
      'PageDown': `goPageDown`,
      'PageUp': `goPageUp`,
      'Right': `goCharRight`,
      'Ctrl-F': `goCharRight`
    };

    this.outputWidgets = [];
    this.currentLine = 0;
    this.submitInProgress = false;

    this.options = options;
    if (!this.options.prompt) {
      this.options.prompt = `> `;
    }
    this.initCallbacks(options);
    this.initializeKeyMap();
    this.initConsole(el);

    this.submitHistory = new KonsoleHistory(this.options);
  }

  refresh() {
    this.console.refresh();
  }

  setValue(value) {
    return this.setLine(this.lineNumber(), `${this.options.prompt}${value}`);
  }

  getValue() {
    return this.getAllInput();
  }

  setLine(lineNumber, content) {
    return this.helpers.setLine(lineNumber, content);
  }

  setPrompt(prompt) {
    this.setLine(this.currentLine,
                 this.console.doc.getLine(this.currentLine)
                 .replace(new RegExp(this.options.prompt), prompt));
    return this.options.prompt = prompt;
  }

  focus() {
    return this.console.getInputField().focus();
  }

  appendToInput(value) {
    return this.setLine(
      this.lineNumber(),
      `${this.console.doc.getLine(this.lineNumber())}${value}`);
  }

  getAllInput() {
    let startingInput;

    startingInput = this.currentLine;
    const input = [];
    while (startingInput <= this.lineNumber()) {
      if (startingInput === this.currentLine) {
        const lineInput = this.console.doc.getLine(startingInput)
                .substr(this.promptLength(),
                        this.console.doc.getLine(this.currentLine).length);
        input.push(lineInput);
      } else {
        input.push(this.console.doc.getLine(startingInput));
      }
      startingInput++;
    }
    return input.join(`\n`);
  }

  reset(welcomeMessage) {
    let _i, _len;

    if (welcomeMessage == null) {
      welcomeMessage = true;
    }
    this.submitInProgress = false;
    this.console.setValue(``);
    this.currentLine = 0;
    const _ref = this.outputWidgets;
    for (_i = 0, _len = _ref.length; _i < _len; _i++) {
      const widget = _ref[_i];
      this.console.removeLineWidget(widget);
    }
    this.outputWidgets = [];
    if (this.options.welcomeMessage && welcomeMessage) {
      this.showWelcomeMessage();
      this.moveInputForward();
    }
    this.console.refresh();
    return this.console.scrollIntoView();
  }

  innerConsole() {
    return this.console;
  }

  initializeKeyMap() {
    // return window.CodeMirror.keyMap.console = this.keyMap;
    return CodeMirror.keyMap.console = this.keyMap;
  }

  initConsole(el) {
    let _this = this;

    _this = this;

    el.className += ` cs-console cs-console-height cs-console-width`;
    const keyActions = new KeyActions(this.options);
    this.console = CodeMirror(el, {
      scrollbarStyle: null,
      mode: {
        name: this.options.syntax,
        useCPP: true
      },
      gutter: this.options.lineNumbers,
      lineNumbers: this.options.lineNumbers,
      theme: this.options.theme || `konsole`,
      indentUnit: 2,
      tabSize: 2,
      keyMap: `console`,
      lineWrapping: true,
      undoDepth: 0,
      autoFocus: this.options.autoFocus,
      extraKeys: {
        'Enter': () => this.submit(),
        'Ctrl-M': () => this.noop(),
        'Tab': () => this.noop(),
        'Left': () => keyActions.goCharLeft(),
        'Ctrl-B': () => keyActions.goCharLeft(),
        'Backspace': () => keyActions.delCharBefore(),
        'Cmd-Up': () => keyActions.goDocStart(),
        'Cmd-Down': () => keyActions.goDocEnd(),
        'Cmd-Left': () => keyActions.goLineStart(),
        'Home': () => keyActions.goLineStart(),
        'Ctrl-A': () => keyActions.goLineStart(),
        'Alt-Backspace': () => keyActions.delGroupBefore(),
        'Ctrl-W': () => keyActions.delGroupBefore(),
        'Cmd-Backspace': () => keyActions.deleteLine(),
        'Up': () => this.nextHistory(),
        'Down': () => this.previousHistory(),
        'Ctrl-P': () => this.nextHistory(),
        'Ctrl-N': () => this.previousHistory(),
        'Shift-Cmd-Right': () => this.noop(),
        'Shift-Cmd-Left': () => this.noop(),
        'Shift-Right': () => this.noop(),
        'Shift-Alt-Right': () => this.noop(),
        'Shift-Alt-Left': () => this.noop(),
        'Ctrl-Enter': () => this.noop(),
        'Alt-Enter': () => this.noop(),
        'Shift-Tab': () => this.noop(),
        'Cmd-S': () => this.noop(),
        'Ctrl-Z': () => this.noop(),
        'Cmd-Z': () => this.noop()
      }
    });
    this.helpers = new CodeMirrorHelpers(this.console);
    keyActions.setConsole(this.console);

    // this.console.on(`keydown`, this.focusInput);

    this.console.on(`keydown`, (cm, evt) => {
      const cursorPos = this.console.getCursor();
      if (cursorPos.line === this.lineNumber()) {
        this.storedCursorPosition = this.console.getCursor();
        if (cursorPos.ch < this.promptLength()) {
          this.console.setCursor({
            line: cursorPos.line,
            ch: this.promptLength()
          });
        }
      } else {
        this.console.setCursor(this.storedCursorPosition);
      }
      return false;
    });

    setTimeout(() => _this.console.refresh(), 1);
    this.console.getScrollerElement().className += ` cs-console-height`;
    this.console.getWrapperElement().className +=
      ` cs-console-height cs-console-width`;
    if (this.options.welcomeMessage) {
      this.showWelcomeMessage();
    }
    if (this.options.initialValue) {
      this.setValue(this.options.initialValue);
      this.moveInputForward();
    }
    if (this.options.autoFocus) {
      setTimeout(() => _this.console.getInputField().focus(), 10);
    }

    return this.moveInputForward();
  }

  focusInput(cm, evt) {
    const cursorPos = this.console.getCursor();
    if (cursorPos.line === this.lineNumber()) {
      this.storedCursorPosition = this.console.getCursor();
      if (cursorPos.ch < this.promptLength()) {
        this.console.setCursor({
          line: cursorPos.line,
          ch: this.promptLength()
        });
      }
    } else {
      this.console.setCursor(this.storedCursorPosition);
    }
    return false;
  }

  nextHistory() {
    return this.setValue(this.submitHistory.nextHistory());
  }

  previousHistory() {
    return this.setValue(this.submitHistory.previousHistory());
  }

  showWelcomeMessage() {
    this.console.setValue(``);
    const line = {
      content: this.options.welcomeMessage
    };
    return this.buildWidget(1, line, {
      above: true
    });
  }

  initCallbacks(options) {
    this.commandValidate = options.commandValidate;
    return this.commandHandle = options.commandHandle;
  }

  submit() {
    const input = this.getAllInput();
    // todo: isg: the undefined was void 0
    if ((this.options.commandValidate === undefined ||
         this.options.commandValidate(input)) && !this.submitInProgress) {
      this.submitInProgress = true;
      this.submitHistory.push(input);
      this.submitHistory.resetIndex();
      return this.commandHandle(input, this.responseObject(),
                                this.options.prompt);
    } else if (this.submitInProgress) {
      return this.nonReactingNewline();
    } else {
      return this.moveInputForward();
    }
  }

  nonReactingNewline() {
    this.currentLine = this.lineNumber();
    return this.setLine(this.currentLine, `${this.inputLine()}\n`);
  }

  promptLength() {
    return this.options.prompt.length;
  }

  inputLine() {
    return this.console.doc.getLine(this.lineNumber());
  }

  lineNumber() {
    return this.console.lineCount() - 1;
  }

  responseObject() {
    const _this = this;

    return function(responseLines) {
      return _this.renderResponse(responseLines);
    };
  }

  renderResponse(responseLines) {
    let _i, _len;

    if (!responseLines) {
      this.moveInputForward();
      this.submitInProgress = false;
      return undefined;
    }
    const lineNumber = this.lineNumber();
    if (responseLines.constructor === Array) {
      for (_i = 0, _len = responseLines.length; _i < _len; _i++) {
        const line = responseLines[_i];
        this.buildWidget(lineNumber, line);
      }
    } else {
      this.buildWidget(lineNumber, responseLines);
    }
    this.buildWidget(lineNumber, {
      content: document.createElement(`p`),
      className: `cs-console-output-spacer bottom`
    });
    this.moveInputForward();
    return this.submitInProgress = false;
  }

  htmlEscape(string) {
    return (`${string}`)
      .replace(/&(?!\w+;|#\d+;|#x[\da-f]+;)/gi, `&amp;`)
      .replace(/</g, `&lt;`)
      .replace(/>/g, `&gt;`)
      .replace(/"/g, `&quot;`)
      .replace(/`/g, `&#x27;`)
      .replace(/\//g, `&#x2F;`);
  }

  buildWidget(lineNumber, responseLine, widgetOptions) {
    let widgetElement;
    const _this = this;

    if (widgetOptions == null) {
      widgetOptions = {};
    }
    const widgetContent = responseLine ? responseLine.content : ``;
    if (this.isHtmlElement(widgetContent)) {
      widgetElement = widgetContent;
    } else {
      widgetElement = document.createElement(`div`);
      widgetElement.innerHTML =
        this.formatWidgetElementText(this.htmlEscape(widgetContent));
      widgetElement.className = `cs-console-output-element`;
      widgetElement.style.whiteSpace = `pre-wrap`;
    }
    // todo: isg: undefined was void 0
    if (responseLine != null ? responseLine.className : undefined) {
      widgetElement.className += ` ${responseLine.className}`;
    }
    if (Object.keys(widgetOptions).indexOf(`coverGutter`) < 0) {
      widgetOptions.coverGutter = false;
    }
    if (Object.keys(widgetOptions).indexOf(`noHScroll`) < 0) {
      widgetOptions.noHScroll = true;
    }
    this.outputWidgets.push(this.console.addLineWidget(lineNumber,
                                                       widgetElement,
                                                       widgetOptions));
    return setTimeout(() => _this.console.scrollIntoView({
      line: _this.console.lineCount() - 1,
      ch: 0
    }), 100);
  }

  isHtmlElement(obj) {
    return obj && obj.constructor.toString().search(/HTML.+Element/) > 0;
  }

  formatWidgetElementText(message) {
    return message.replace(/^\s/, ``);
  }

  moveInputForward() {
    this.currentLine = this.lineNumber() + 1;


    this.setLine(this.currentLine - 1,
                 `${this.inputLine()}\n${this.options.prompt}`);
    this.storedCursorPosition = {
      line: this.currentLine,
      ch: this.promptLength()
    };
    return this.console.setCursor(this.storedCursorPosition);
  }

  noop() {}

  log(text) {
    this.appendToInput(text);
    this.moveInputForward();
  }
}

/* eslint-enable no-unused-vars */
/* ------------------------------------------------------------ */
