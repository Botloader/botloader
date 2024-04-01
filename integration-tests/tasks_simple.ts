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

script.onTask<Data>("simple", (t) => {
    console.log("got task!", t.id);

    assertElapsed(t.data.scheduledAt, 10000);

    data.scheduledAt = t.data.scheduledAt;
    assetJsonEquals(t.data, data);

    sendScriptCompletion(script.name);
})

runOnce(script.name, async () => {
    await Tasks.schedule("simple", new Date(data.scheduledAt + 10000), {
        data: data
    });
})

// // Example: Required data
// const bucketRequired = script.createTaskBucket<string>({
//     namespace: "funtimes",
// }, (task) => {
//     const data = task.data
// })

// bucketRequired.schedule({ execute_at: new Date(), data: "hello" })
// // error because incorrect type
// bucketRequired.schedule({ execute_at: new Date(), data: 10 })
// // error because no data provided
// bucketRequired.schedule({ execute_at: new Date() })


// // Example: No data
// const bucketNoData = script.createTaskBucket({
//     namespace: "funtimes",
// }, (task) => {
//     const data = task.data
// })

// bucketNoData.schedule({ execute_at: new Date() })
// // Error because incorrect type (this task bucket has no data)
// bucketNoData.schedule({ execute_at: new Date(), data: "" })

// // Example: optional data
// const buckeOptionalData = script.createTaskBucket<string | undefined>({
//     namespace: "funtimes",
// }, (task) => {
//     const data = task.data
// })

// buckeOptionalData.schedule({ execute_at: new Date(), data: "gaming" })
// // Valid because this is optional data
// buckeOptionalData.schedule({ execute_at: new Date() })