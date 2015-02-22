/*
  Copyright 2015 Inderjit Gill

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/

export class PublicBinding {
  constructor(name, doc, create) {
    this.name = name;
    this.doc = doc;
    this.create = create;
  }
}

export function bind(env, objs) {
  // objs is an array
  // add every key,val pair in obj as a binding to env
  let bindAll = function(obj) {
    for(let key in obj) {
      env.add(key, obj[key])
    }
  };
  
  objs.forEach(o => bindAll(o));
  return env;
}

export class Env {

  constructor(outer = null) {
    this.outer = outer;
    this.bindings = new Map();
  }

  newScope() {
    return new Env(this);
  }

  add(key, val) {
    this.bindings.set(key, val);
    return this;
  }

  hasBinding(key) {
    let e = this;
    while(e !== null) {
      if(e.hasBindingInThisScope(key)) {
        return true;
      }
      e = e.outer;
    }
    return false;
  }

  hasBindingInThisScope(key) {
    return this.bindings.has(key);
  }

  lookup(key) {
    let e = this;
    while(e !== null) {
      if(e.hasBindingInThisScope(key)) {
        return e.bindings.get(key);
      }
      e = e.outer;
    }
    return undefined;           // ???
  }

  mutate(key, val) {
    let e = this;
    while(e !== null) {
      if(e.hasBindingInThisScope(key)) {
        e.bindings.set(key, val);
        return this;
      }
      e = e.outer;
    }

    // the key did not exist in this env
    // so create a new binding
    return this.add(key, val);
  }
}