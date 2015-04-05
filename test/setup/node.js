/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

global.chai = require('chai');
global.sinon = require('sinon');
global.chai.use(require('sinon-chai'));

global.Immutable = require('immutable');
global.Math.seedrandom = require('seedrandom');

global.glmatrix = require('gl-matrix');
global.mat4 = global.glmatrix.mat4;
global.vec3 = global.glmatrix.vec3;

require('babel/register');
require('./setup')();
