import { Storage } from "botloader";
import { assertElapsed, sendScriptCompletion } from "lib"

let store = script.createGuildStorageNumber("interval_2");

script.onInterval("test2", "* * * * *", async () => {
    let lastRun = await store.get("last_run");
    if (lastRun) {
        assertElapsed(lastRun.value, 60000);
        sendScriptCompletion();
    } else {
        await store.set("last_run", Date.now());
    }
})