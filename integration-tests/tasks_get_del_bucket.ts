import { Tasks } from "botloader";
import { assertExpectError, assertExpected, assetJsonEquals, runOnce, sendScriptCompletion } from "lib"

const bucket = script.createTaskBucket({
    name: "tasks_get_del_bucket",
}, () => { })

const anotherBucket = script.createTaskBucket({
    name: "tasks_get_del_bucket_2",
}, () => { })

const bucketPluginCustom = script.createTaskBucket({
    name: "tasks_get_del_bucket",
    customScope: { kind: "Plugin", pluginId: "1" }
}, () => { })

runOnce("tasks_get_del_bucket.ts", async () => {
    await testGetDelKeys()
    await testGetAll()
    await testDelSingular()

    sendScriptCompletion()
})

async function testGetDelKeys() {
    let task1 = await bucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });
    let task2 = await bucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "2" });
    let taskOtherNs = await anotherBucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });
    let taskOtherPlugin = await bucketPluginCustom.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });

    assetJsonEquals(await Tasks.getById(task1.id), task1);
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);
    assetJsonEquals(await Tasks.getById(taskOtherPlugin.id), taskOtherPlugin);

    assetJsonEquals(await bucket.getByKey("1"), task1);
    assetJsonEquals(await bucket.getByKey("2"), task2);
    assetJsonEquals(await anotherBucket.getByKey("1"), taskOtherNs);
    assetJsonEquals(await bucketPluginCustom.getByKey("1"), taskOtherPlugin);

    // test deleteAll while cleaning up

    // deleting the guild scoped bucket and ensuring it only deleted that
    await bucket.deleteAll()
    assertExpected(undefined, await bucket.getByKey("1"))
    assertExpected(undefined, await bucket.getByKey("2"))

    assetJsonEquals(taskOtherNs, await anotherBucket.getByKey("1"));
    assetJsonEquals(taskOtherPlugin, await bucketPluginCustom.getByKey("1"));

    // deleting plugin scoped bucket and ensuring it did not touch the guild one
    await bucketPluginCustom.deleteAll()
    assetJsonEquals(taskOtherNs, await anotherBucket.getByKey("1"));
    assertExpected(undefined, await bucketPluginCustom.getByKey("1"));

    await anotherBucket.deleteAll()
    assertExpected(undefined, await anotherBucket.getByKey("1"));
}

async function testGetAll() {
    for (let i = 0; i < 55; i++) {
        await bucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "" + i });
        await anotherBucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "100" + i });
        await bucketPluginCustom.schedule({ executeAt: new Date(Date.now() + 1000000), key: "10000" + i });
    }

    const inNamespace = await countAll(bucket);
    assertExpected(55, inNamespace)

    const inPlugin = await countAll(bucketPluginCustom);
    assertExpected(55, inPlugin)

    const all = await countAll();
    // other scripts are running, so we we have no idea how many there is overall, but it should be atleast 200
    if (all < 110) {
        throw new Error(`${inNamespace} < 110`)
    }

    await bucket.deleteAll()
    await anotherBucket.deleteAll()
    await bucketPluginCustom.deleteAll()
}

async function countAll(bucket?: Tasks.TaskBucket): Promise<number> {
    let last = 0;
    let n = 0;
    while (true) {
        const batch = bucket
            ? await bucket.getMany({
                afterId: last,
            })
            : await Tasks.getMany({
                afterId: last,
            });

        if (batch && batch.length > 0) {
            last = batch[batch.length - 1].id;
            n += batch.length;
        } else {
            break;
        }
    }

    return n
}

async function testDelSingular() {
    let task1 = await bucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });
    let task2 = await bucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "2" });
    let taskOtherNs = await anotherBucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });
    let taskOtherPlugin = await bucketPluginCustom.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });

    await bucket.deleteById(task1.id)

    assertExpected(undefined, await Tasks.getById(task1.id));
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);
    assetJsonEquals(await Tasks.getById(taskOtherPlugin.id), taskOtherPlugin);

    // pass! on to next scenario

    task1 = await bucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });

    await bucket.deleteByKey("1")

    assertExpected(undefined, await Tasks.getById(task1.id));
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);
    assetJsonEquals(await Tasks.getById(taskOtherPlugin.id), taskOtherPlugin);

    // pass! on to next scenario

    task1 = await bucket.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });

    await bucketPluginCustom.deleteById(taskOtherPlugin.id)

    assetJsonEquals(await Tasks.getById(task1.id), task1);
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);
    assertExpected(undefined, await Tasks.getById(taskOtherPlugin.id));

    // pass! on to next scenario

    taskOtherPlugin = await bucketPluginCustom.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });
    await bucketPluginCustom.deleteByKey("1")

    assetJsonEquals(await Tasks.getById(task1.id), task1);
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);
    assertExpected(undefined, await Tasks.getById(taskOtherPlugin.id));

    // pass! on to next scenario
    taskOtherPlugin = await bucketPluginCustom.schedule({ executeAt: new Date(Date.now() + 1000000), key: "1" });

    // finally test if deleting another bucket fails
    await assertExpectError(async () => {
        await bucket.deleteById(taskOtherNs.id)
    })

    await assertExpectError(async () => {
        await bucket.deleteById(taskOtherPlugin.id)
    })

    await assertExpectError(async () => {
        await bucketPluginCustom.deleteById(task1.id)
    })
}