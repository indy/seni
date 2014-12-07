import {
  Env
} from '../../src/lang/env';

describe('env', function () {

  var e;
  var key;
  var val;

  beforeEach(function () {
    e = new Env();

    key = "number";
    val = 5;

    e.addBinding(key, val);
  });


  it('should add a new binding', function () {
    expect(e.hasBindingInThisScope(key)).toBe(true);
    expect(e.hasBindingInThisScope("other")).toBe(false);
  });

  it('should lookup a binding', function () {
    expect(e.lookup(key)).toEqual(val);
  });  

  it('should be able to resolve bindings using outer scopes', function () {
    let e2 = new Env(e);

    // key is not bound directly to e2
    expect(e2.hasBindingInThisScope(key)).toBe(false);
    // but binding can still be resolved from e2
    expect(e2.hasBinding(key)).toBe(true);
  });

  it('should be able to lookup using outer scopes', function () {
    let e2 = new Env(e);
    expect(e2.lookup(key)).toEqual(val);
  });

  it('should be able to use latest binding', function () {
    let laterVal = 42,
        e2 = new Env(e);

    e2.addBinding(key, laterVal);
    
    expect(e2.lookup(key)).toEqual(laterVal);
  });  

});
