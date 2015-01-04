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
