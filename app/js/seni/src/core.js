import { PublicBinding } from 'lang/env';

export const takeBinding = new PublicBinding(
  "take",

  `invokes the 'from' function 'num' times, returning a list`,

  () => function({num = 0, from = () => 0}) {
    let res = [];
    for(let i=0; i<num; i++) {
      res.push(from());
    }
    return res;
  }
)
