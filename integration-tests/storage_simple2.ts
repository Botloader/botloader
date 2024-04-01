import { Storage } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

runOnce(script.name, async () => {
    let bucket = script.createStorageNumber("storage_simple2.ts");

    await assertCount(bucket, 0);

    await bucket.set("t_a_1", 1);
    await bucket.set("t_a_2", 1);
    await bucket.set("t_b_1", 1);
    await bucket.set("t_b_2", 1);
    await bucket.set("taa_2", 1);
    await assertCount(bucket, 5);
    await assertCount(bucket, 0, "asd");
    await assertCount(bucket, 4, "t\\_%");
    await assertCount(bucket, 2, "t\\_a\\_%");
    await assertCount(bucket, 5, "t_%");

    let deleted = await bucket.deleteAll("t\\_a%");
    assertExpected(2, deleted)
    await assertCount(bucket, 3);

    sendScriptCompletion(script.name);
});

async function assertCount<T>(bucket: Storage.Bucket<T>, expected: number, pattern?: string) {
    let count = await bucket.count(pattern);

    if (count !== expected) {
        throw new Error(`Bucket count of ${bucket.name} was ${count} but expected ${expected}`);
    }
}