import { Storage } from "botloader";
import { sendScriptCompletion } from "lib"

let store = script.createGuildStorageJson("interval_1");
script.onInterval("interval_1", "* * * * *", async () => {
    if (await store.setIf("has_run", true, "IfNotExists")) {
        sendScriptCompletion();
    }
})