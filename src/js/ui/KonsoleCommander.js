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

export default class KonsoleCommander {
  constructor() {
    this.commands = [];
  }

  // add a command to the konsole
  // a konsoleCommand is an object that has the following functions:
  //   canHandle(command: string) => boolean
  //   execute(command: string) => string
  //
  addCommand(konsoleCommand) {
    this.commands.push(konsoleCommand);
  }

  // todo: prompt is also passed in
  commandHandle(line, report) {
    const words = line.split(' ');
    const commandName = words[0].trim();

    try {
      const command = this.commands.find(c => c.canHandle(commandName));
      if (command) {
        const args = words.slice(1);
        report({content: command.execute(commandName, args)});
      } else {
        report({content: `unknown command: ${line}`});
      }
    } catch (e) {
      report({content: `caught exception: ${e}`});
    }
  }
}
