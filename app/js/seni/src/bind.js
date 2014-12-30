import { renderBezier, getBezierFn } from './Bezier';
import { Env } from 'lang/env';
import { Node, NodeType } from 'lang/node';

export function addBindings(env, renderer) {

  var glContainer = renderer.getGLContainer();
  var buffer = renderer.getBuffer();
  var bezier = getBezierFn(glContainer, buffer);

  env.addBinding('bezier', function(args) {

    let bargs = {};
    
    if(args.length > 0) {
      bargs.tesselation = args[0];
    }
    if(args.length > 1) {
      bargs.lineWidth = args[1];
    }
    if(args.length > 2) {
      bargs.coords = args[2];
    }
    if(args.length > 3) {
      bargs.tStart = args[3];
    }
    if(args.length > 4) {
      bargs.tEnd = args[4];
    }

    bezier(bargs);
/*
    let h = args[0];
    console.log(h);
    bezier({coords: [[440, 200 + h],
                     [533, 500 + h],
                     [766, 0 + h],
                     [900, 300 + h]]});
*/
  });

  return env;
}
