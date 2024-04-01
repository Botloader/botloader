import { Discord, Tasks } from "botloader";
import { runOnce, sendScriptCompletion } from "lib";

runOnce(script.name, async () => {
    for (let i = 0; i < 15; i++) {
        try {
            await Discord.createMessage("959117510161076266", { content: "invalid test" });
        } catch (e) { }

    }

    await Tasks.schedule("bad_requests_send_more", new Date(Date.now() + (61 * 1000)))
})

script.onTask("bad_requests_send_more", async (t) => {
    for (let i = 0; i < 15; i++) {
        try {

            await Discord.createMessage("959117510161076266", { content: "invalid test" });
        } catch (e) { }
    }

    sendScriptCompletion(script.name);
})