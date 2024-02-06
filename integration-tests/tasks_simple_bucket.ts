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

const bucket = script.createTaskBucket<Data>({
    name: "simple_bucket",
}, (t) => {
    console.log("got task!", t.id);

    assertElapsed(t.data.scheduledAt, 10000);

    data.scheduledAt = t.data.scheduledAt;
    assetJsonEquals(t.data, data);

    sendScriptCompletion();
})

runOnce("tasks_simple_bucket.ts", async () => {
    await bucket.schedule({
        executeAt: new Date(data.scheduledAt + 10000),
        data,
    })
})
