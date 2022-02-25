import { Tasks } from "botloader";
import { runOnce, sendScriptCompletion } from "lib"

// this test checks the validation of the max value size
runOnce("tasks_data.ts", async () => {

    let data: any[] | null = [];
    for (let i = 0; i < 1000; i++) {
        data.push("0123456789")
    }

    try {
        await Tasks.schedule("data", new Date(Date.now() + 10000), {
            data: data
        });
    } catch (e) {
        data = null;
        sendScriptCompletion();
        return;
    }

    throw new Error("expected an error, data validation failed")
})