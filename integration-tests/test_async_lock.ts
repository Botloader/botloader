import { AsyncLock, Discord, Storage } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

runOnce(script.name, async () => {

    const lock = new AsyncLock()
    lock.withLocked(whileLocked)
    lock.withLocked(whileLocked)

    sendScriptCompletion(script.name);

});

let someVar = 10
async function whileLocked() {
    let copiedVar = someVar
    someVar += 10

    // emulate an async operation
    await Discord.getMember(Discord.getBotUser().id)

    assertExpected(someVar, copiedVar + 10)
    someVar = copiedVar + 20

    await Discord.getMember(Discord.getBotUser().id)

    // ensure it hasn't changed
    assertExpected(someVar, copiedVar + 20)
    someVar = copiedVar + 30
}