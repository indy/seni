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
import Trivia from '../seni/Trivia';

export function addDefaultCommands(atom, commander) {

  const ls = {
    canHandle(command) {
      return command === 'ls';
    },

    execute() {
      const app = atom.app;
      const env = app.get('env');
      const keys = env.keys();

      const res = [];
      for (let k = keys.next(); k.done === false; k = keys.next()) {
        res.push(k.value);
      }
      return res.sort().join('\n');
    }
  };

  const title = {
    canHandle(command) {
      return command === 'title';
    },

    execute() {
      return Trivia.getTitle();
    }
  };

  const help = {
    canHandle(command) {
      return command === 'help';
    },

    execute(_, [name, showDefaultArgs]) {
      // todo: if no args given then show generic help for the konsole
      const app = atom.app;
      const v = app.getIn(['env', name]);

      let res = '';

      if (v.pb) {
        const binding = v.pb;       // publicBinding
        res = `${name}: ${binding.doc}`;
        if (showDefaultArgs) {
          const args = JSON.stringify(binding.defaults, null, ' ');
          res = `${res}\ndefault arguments ${args}`;
        }
      }
      return res;
    }
  };

  [ls, title, help].forEach(c => commander.addCommand(c));
}

