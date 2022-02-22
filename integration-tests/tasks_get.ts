import { Tasks } from "botloader";
import { assetJsonEquals, runOnce, sendScriptCompletion } from "lib";

async function testGetKey() {
    let task1 = await Tasks.schedule("get_key", new Date(Date.now() + 1000000), { key: "1" });
    let task2 = await Tasks.schedule("get_key", new Date(Date.now() + 1000000), { key: "2" });
    let taskOtherNs = await Tasks.schedule("get_key2", new Date(Date.now() + 1000000), { key: "1" });

    assetJsonEquals(await Tasks.getById(task1.id), task1);
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);

    assetJsonEquals(await Tasks.getByKey("get_key", "1"), task1);
    assetJsonEquals(await Tasks.getByKey("get_key", "2"), task2);
    assetJsonEquals(await Tasks.getByKey("get_key2", "1"), taskOtherNs);
}

async function testGetAll() {
    for (let i = 0; i < 55; i++) {
        await Tasks.schedule("get_many", new Date(Date.now() + 1000000), { key: "" + i });
        await Tasks.schedule("get_many2", new Date(Date.now() + 1000000), { key: "" + i });
    }

    const inNamespace = await countAll("get_many");
    if (inNamespace !== 55) {
        throw new Error(`${inNamespace} !== 55`)
    }

    const all = await countAll();
    // other scripts are running, so we we have no idea how many there is overall, but it should be atleast 200
    if (all < 110) {
        throw new Error(`${inNamespace} < 110`)
    }
}

async function countAll(namespace?: string): Promise<number> {
    let last = 0;
    let n = 0;
    while (true) {
        const batch = await Tasks.getMany({
            namespace: namespace, afterId: last
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
runOnce("tasks_get.ts", async () => {
    await testGetKey();
    await testGetAll();

    sendScriptCompletion();
})