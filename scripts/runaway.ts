import { } from 'botloader';

let a = 0;
let b = a;

script.on("MESSAGE_CREATE", async evt => {
    while (true) {
        a++;
        b = a;
    }
})
