import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const counter = script.createStorageVarNumber("storage_var_counter");

runOnce("storage_var.ts", async () => {

    const entry = await counter.incr(1);
    const anotherEntry = await counter.incr(1);

    assertExpected(entry.value + 1, anotherEntry.value);

    await counter.set(100);
    let changed = await counter.get();
    assertExpected(100, changed?.value);

    sendScriptCompletion();
});
