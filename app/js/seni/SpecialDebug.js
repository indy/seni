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

import PublicBinding from '../lang/PublicBinding';
import Interpreter from '../lang/Interpreter';

const { evaluate, TRUE_STRING, FALSE_STRING, NO_ERROR } = Interpreter;

let gKonsole = undefined;

function useKonsole(konsole) {
  gKonsole = konsole;
}

const publicBindings = [
  new PublicBinding(
    'print',
    { description: "(print 'hi' foo) => hi 42",
      args: [],
      returns: '-' },
    {},
    _self => (env, [_, ...msgs]) => {
      const printMsg = msgs.reduce((a, b) => `${a} ${evaluate(env, b)[1]}`, '');
      if (gKonsole) {
        gKonsole.log(printMsg.trim());
      }
      return [env, true, NO_ERROR];
    }
  ),

  new PublicBinding(
    'log',
    { description: "(log 'hi' foo) => hi <foo:42>",
      args: [],
      returns: '-' },
    {},
    _self => (env, [_, ...msgs]) => {
      let firstError = NO_ERROR;
      const message = msgs.reduce((a, b) => {
        const [_e, res, err] = evaluate(env, b);
        if (err && firstError === NO_ERROR) {
          firstError = err;
        }
        if (typeof b === 'string' && b !== TRUE_STRING && b !== FALSE_STRING) {
          return `${a} < ${b}:${res}>`;
        }
        return `${a} ${res}`;
      }, '');
      if (gKonsole) {
        gKonsole.log(message);
      }
      return [env, true, firstError];
    }
  )
];

export default {
  publicBindingType: 'special',
  publicBindings,
  useKonsole
};
