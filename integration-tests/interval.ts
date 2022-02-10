import { Storage } from "botloader";
import { sendScriptCompletion } from "lib"

let store = script.registerStorageBucket(new Storage.JsonBucket("interval_1"));
script.registerIntervalTimer("interval_1", "* * * * *", async () => {
    if (await store.setIf("has_run", true, "IfNotExists")) {
        sendScriptCompletion();
    }
})