import Util from '../seni/Util';

class PublicBinding {
  constructor(name, doc, defaults, create) {
    this.name = name;
    this.doc = doc;
    this.defaults = defaults;
    this.create = create;
  }

  mergeWithDefaults(params) {
    return Util.merge(params, this.defaults);
  }
}

export default PublicBinding;
