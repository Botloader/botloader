import { Tasks } from "botloader";
import { assertElapsed, assetJsonEquals, runOnce, sendScriptCompletion } from "lib"

interface Data {
    someString: string,
    someNumber: number,
    scheduledAt: number,

};

let data: Data = {
    someNumber: 1000,
    someString: "hello there",
    scheduledAt: Date.now(),

}

script.registerTaskHandler<Data>("simple", (t) => {
    console.log("got task!", t.id);

    assertElapsed(t.data.scheduledAt, 10000);

    data.scheduledAt = t.data.scheduledAt;
    assetJsonEquals(t.data, data);

    sendScriptCompletion();
})

runOnce("tasks_simple.ts", async () => {
    await Tasks.scheduleTask("simple", new Date(data.scheduledAt + 10000), {
        data: data
    });
})