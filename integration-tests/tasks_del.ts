import { Tasks } from "botloader";
import { assetJsonEquals, runOnce, sendScriptCompletion } from "lib";

async function testDelId() {
    let task = await Tasks.scheduleTask("del_id", new Date(Date.now() + 1000000));
    assetJsonEquals(await Tasks.getById(task.id), task);

    if (!await Tasks.deleteById(task.id)) {
        throw new Error("failed deleting task");
    }

    if (await Tasks.getById(task.id) !== undefined) {
        throw new Error("task not deleted");
    }
}


async function testDelKey() {
    let task1 = await Tasks.scheduleTask("del_key", new Date(Date.now() + 1000000), { key: "1" });
    let task2 = await Tasks.scheduleTask("del_key", new Date(Date.now() + 1000000), { key: "2" });
    let taskOtherNs = await Tasks.scheduleTask("del_key2", new Date(Date.now() + 1000000), { key: "1" });

    assetJsonEquals(await Tasks.getById(task1.id), task1);
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);

    if (!await Tasks.deleteByKey("del_key", "1")) {
        throw new Error("failed deleting task");
    }

    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);
}

async function testDelNamespace() {
    let task1 = await Tasks.scheduleTask("del_ns", new Date(Date.now() + 1000000), { key: "1" });
    let task2 = await Tasks.scheduleTask("del_ns", new Date(Date.now() + 1000000), { key: "2" });
    let taskOtherNs = await Tasks.scheduleTask("del_ns2", new Date(Date.now() + 1000000), { key: "1" });

    assetJsonEquals(await Tasks.getById(task1.id), task1);
    assetJsonEquals(await Tasks.getById(task2.id), task2);
    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);

    const n = await Tasks.deleteNamespace("del_ns");
    if (n !== 2) {
        throw new Error(`${n} !== 2`);
    }

    assetJsonEquals(await Tasks.getById(taskOtherNs.id), taskOtherNs);
}

runOnce("tasks_del.ts", async () => {
    await testDelId();
    await testDelKey();
    await testDelNamespace();

    sendScriptCompletion();
})