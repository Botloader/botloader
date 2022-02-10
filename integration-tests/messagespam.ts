import { runOnce, sendScriptCompletion } from "lib";

runOnce("messagespam.ts", async () => {
    (async () => {
        for (let i = 0; i < 10; i++) {
            await script.createMessage("531120790318350338", { content: `testing.... ${i}` })
        }

        sendScriptCompletion();
    })();
})