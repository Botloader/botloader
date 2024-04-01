import { Storage } from "botloader";
import { assertElapsed, sendScriptCompletion } from "lib"

let completed = script.createStorageVarNumber("interval_2_completed");
let lastRun = script.createStorageVarNumber("interval_2_last");

script.onInterval("test2", "* * * * *", async () => {
    if (await completed.get()) {
        return;
    }

    let lr = await lastRun.get()
    if (lr) {
        assertElapsed(lr.value, 60000);
        await completed.set(1);
        sendScriptCompletion(script.name);
    } else {
        await lastRun.set(Date.now());
    }
})