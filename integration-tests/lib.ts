type TestingEvent = "ScriptComplete";

const run_tracker = script.createStorageJson("lib_run_tests");

export async function sendScriptCompletion(name: String) {
    if (await run_tracker.setIf("completed_" + name, true, "IfNotExists")) {
        sendTestingEvent("ScriptComplete");
        console.log(`${name} completed`)
    } else {
        throw new Error(`Tried to complete test twice: ${name}`)
    }
}

export function sendTestingEvent(event: TestingEvent) {
    console.log(`INTEGRATION_TEST:${JSON.stringify(event)}`)
}

export function assertElapsed(since: number, expected: number, error?: number) {
    const _error = error ?? 1000

    let elapsed = Date.now() - since;
    if (elapsed > expected + _error) {
        throw new Error(`elapsed time (${elapsed}) significantly greater than expected (${expected})`);
    } else if (elapsed < expected - _error) {
        throw new Error(`elapsed time (${elapsed}) significantly less than expected (${expected})`);
    }
}
export function assetJsonEquals(a: any, b: any) {
    let as = JSON.stringify(a);
    let bs = JSON.stringify(b);

    if (bs !== as) {
        throw new Error(`mismatched json data: ${as} !== ${bs}`);
    }
}

export function assertExpected(expected: any, actual: any) {
    if (expected !== actual) {
        throw new Error(`expected: ${expected}, actual: ${actual}`);
    }
}

export async function assertExpectError(f: () => any) {
    let gotError = false
    try {
        await f()
    } catch (error) {
        gotError = true
        console.log("Got expecte error", error)
    }

    if (!gotError) {
        throw new Error("Did not get expected error")
    }
}

export async function runOnce(name: string, cb: () => any) {
    if (await run_tracker.setIf("run_" + name, true, "IfNotExists")) {
        console.log(`Running ${name}`)
        await cb();
    } else {
        console.log("skipping already run test");
    }
}

runOnce(script.name, async () => {
    await sendScriptCompletion(script.name);
})
