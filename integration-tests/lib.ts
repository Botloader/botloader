import { Storage } from "botloader";

type TestingEvent = "ScriptComplete";

export function sendScriptCompletion() {
    sendTestingEvent("ScriptComplete");
}

export function sendTestingEvent(event: TestingEvent) {
    console.log(`INTEGRATION_TEST:${JSON.stringify(event)}`)
}

sendScriptCompletion();

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

let run_tracker = script.createGuildStorageJson("lib_run_tests");

export async function runOnce(name: string, cb: () => any) {
    if (await run_tracker.setIf("run_" + name, true, "IfNotExists")) {
        await cb();
    } else {
        console.log("skipping already run test");
    }
}