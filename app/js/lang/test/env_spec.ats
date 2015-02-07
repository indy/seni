import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import {
  PublicBinding,
  Env
} from 'lang/env';

export function main() {

  describe('env', () => {

    var e;
    var key;
    var val;

    beforeEach(() => {
      e = new Env();

      key = "number";
      val = 5;

      e.add(key, val);
    });


    it('should add a new binding', () => {
      expect(e.hasBindingInThisScope(key)).toBe(true);
      expect(e.hasBindingInThisScope("other")).toBe(false);
    });

    it('should lookup a binding', () => {
      expect(e.lookup(key)).toEqual(val);
    });  

    it('should be able to resolve bindings using outer scopes', () => {
      let e2 = new Env(e);

      // key is not bound directly to e2
      expect(e2.hasBindingInThisScope(key)).toBe(false);
      // but binding can still be resolved from e2
      expect(e2.hasBinding(key)).toBe(true);
    });

    it('should be able to lookup using outer scopes', () => {
      let e2 = new Env(e);
      expect(e2.lookup(key)).toEqual(val);
    });

    it('should be able to use latest binding', () => {
      let laterVal = 42,
      e2 = new Env(e);

      e2.add(key, laterVal);
      
      expect(e2.lookup(key)).toEqual(laterVal);
    });  

    it('should mutate bindings', () => {
      e.mutate(key, 3);
      expect(e.lookup(key)).toEqual(3);
    });  

  });
}
