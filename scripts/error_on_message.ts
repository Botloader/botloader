import { } from 'botloader';

script.on("MESSAGE_CREATE", async evt => {
    if (!evt.author.bot) {
        throw new Error("woo fancy error appeared");
    }
})

