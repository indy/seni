import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import * as shabba from 'core/compiler/shabba';

export function main() {
    describe("Shabba", () => {
        var log, zone;

        beforeEach(() => {
        });

        describe("run", () => {
            it('should call onTurnStart and onTurnDone', () => {
                expect(shabba.twice(21)).toEqual(42);
            });

            it('should return the body return value from run', () => {
                expect(shabba.thrice(33)).toEqual(99);
            });
        });

        describe("runOutsideAngular", () => {
            it("should run a function outside of the angular zone", () => {
                expect(shabba.twice(33)).toEqual(66);
            });
        });
    });
}
