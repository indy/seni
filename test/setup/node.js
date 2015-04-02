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
