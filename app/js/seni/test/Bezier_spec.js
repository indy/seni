import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import {Bezier} from 'seni/Bezier';

export function main() {
    describe('Bezier', () => {
        var bezier;

        beforeEach(() => {
            bezier = new Bezier();
        });

        it('should double', () => {
            expect(bezier.doubler(3)).toEqual(6);
        });

        it('should double again', () => {
            expect(bezier.doubler(3)).toEqual(6);
        });

    });
}
