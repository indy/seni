import Env from '../../src/lang/Env';

describe('env', () => {

  var e;
  var key;
  var val;

  beforeEach(() => {
    e = new Env();

    key = 'number';
    val = 5;

    e.add(key, val);
  });


  it('should add a new binding', () => {
    expect(e.hasBindingInThisScope(key)).to.be.true;
    expect(e.hasBindingInThisScope('other')).to.be.false;
  });

  it('should lookup a binding', () => {
    expect(e.lookup(key)).to.equal(val);
  });

  it('should be able to resolve bindings using outer scopes', () => {
    let e2 = new Env(e);

    // key is not bound directly to e2
    expect(e2.hasBindingInThisScope(key)).to.be.false;
    // but binding can still be resolved from e2
    expect(e2.hasBinding(key)).to.be.true;
  });

  it('should be able to lookup using outer scopes', () => {
    let e2 = new Env(e);
    expect(e2.lookup(key)).to.equal(val);
  });

  it('should be able to use latest binding', () => {
    let laterVal = 42,
        e2 = new Env(e);

    e2.add(key, laterVal);

    expect(e2.lookup(key)).to.equal(laterVal);
  });
});
