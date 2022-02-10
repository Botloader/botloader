import { OpWrappers } from 'botloader';

let counter = 1;

script.on("MESSAGE_CREATE", async evt => {
    if (!evt.author.bot && evt.content === "pog") {
        counter++;
        counter++;
        await OpWrappers.createMessage({
            channelId: evt.channelId,
            content: "pog #" + counter,
        })
    }
})
