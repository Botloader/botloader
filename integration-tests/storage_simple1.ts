import { Storage } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

runOnce("storage_simple1", async () => {
    let bucket = script.createGuildStorageNumber("storage_simple1.ts");

    await assertCount(bucket, 0);

    await bucket.set("t1", 1);
    await bucket.set("t2", 2);
    await bucket.set("t3", 1);
    await assertCount(bucket, 3);

    let deleted = await bucket.deleteAll();
    assertExpected(3, deleted);

    await assertCount(bucket, 0);

    sendScriptCompletion();

});

async function assertCount<T>(bucket: Storage.Bucket<T>, expected: number) {
    let count = await bucket.count();

    if (count !== expected) {
        throw new Error(`Bucket count of ${bucket.name} was ${count} but expected ${expected}`);
    }
}