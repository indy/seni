export class Env {

    constructor(outer = null) {
        this.outer = outer;
        this.bindings = new Map();
    }

    newScope() {
        return new Env(this);
    }

    addBinding(key, val) {
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
}