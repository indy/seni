
export class MathUtil {
    doubler(d) {
        return d * 2;
    }

    foofoo(a, b, ...rest) {
        return a + b;
    }


    stepsInclusive(start, end, num) {
        var unit = (end - start) / (num - 1);
        var res = [];
        for(var i=0;i<num;i++) {
            res.push(start + (i * unit));
        }
        return res;
    }
};

export function stepsInclusive(start, end, num) {
    var unit = (end - start) / (num - 1);
    var res = [];
    for(var i=0;i<num;i++) {
        res.push(start + (i * unit));
    }
    return res;
}




