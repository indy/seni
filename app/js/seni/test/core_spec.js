//import {describe, expect} from 'test_lib/test_lib';

import * as core from 'seni/core';

export function main() {
  describe('core functions', () => {

    it('take', () => {

      let takeFn = core.takeBinding.create();

      let c = 0
      let res = takeFn({num: 3, from: () => c++});

      expect(res.length).toEqual(3);
      for(let i = 0; i < res.length; i++) {
        expect(res[i]).toEqual(i);
      }
    });

  });
}
