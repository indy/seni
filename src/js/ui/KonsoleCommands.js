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
import Trivia from '../sen/Trivia';

export function addDefaultCommands(documentation, commander) {
  const ls = {
    canHandle(command) {
      return command === 'ls';
    },

    execute() {
      const res = [];
      for (const k in documentation) {
        res.push(k);
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

    execute(_, [name, _showDefaultArgs]) {
      // todo: if no args given then show generic help for the konsole
      const v = documentation[name];

      if (!v) {
        return undefined;
      }

      function makeDoc({description, args, returns}) {
        let res = name;
        if (description && description.length > 0) {
          res += `: ${description}`;
        }
        if (args && args.length > 0) {
          res = args.reduce((a, [nm, desc]) => `${a}  ${nm}: ${desc}\n`,
                            `${res}\n\nArguments:\n`);
        }
        if (returns && returns.length > 0) {
          res += `\nReturns: ${returns}`;
        }

        return res;
      }

      const res = makeDoc(v);
      /*
      if (showDefaultArgs) {
        const args = JSON.stringify(binding.defaults, null, ' ');
        res = `${res}\ndefault arguments ${args}`;
      }
      */
      return res;
    }
  };

  [ls, title, help].forEach(c => commander.addCommand(c));
}
