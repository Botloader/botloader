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

script.onTask<Data>("simple_1m", (t) => {
    assertElapsed(t.data.scheduledAt, 60000);

    data.scheduledAt = t.data.scheduledAt
    assetJsonEquals(t.data, data);

    sendScriptCompletion();
})

runOnce("tasks_simple_1m.ts", async () => {
    await Tasks.schedule("simple_1m", new Date(data.scheduledAt + 60000), {
        data: data
    });
}) 