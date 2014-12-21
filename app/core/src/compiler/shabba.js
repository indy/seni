import {doubler} from './ranks';

export function twice(x) {
    return doubler(x);
}

export function thrice(x) {
    return twice(x) + x;
}

