// import { Script } from "botloader"
import { assertExpected, runOnce, sendScriptCompletion } from "lib"

// @ts-ignore
const scriptPlugin: typeof script = new Script(script.scriptId, "1")

interface TestStorageType {
    a: number,
    b: string,
}

const bucketJson = script.createStorageJson<TestStorageType>("storagev2_json")
const bucketJsonPlugin = scriptPlugin.createStorageJson<TestStorageType>("storagev2_json")

const bucketNumber = script.createStorageNumber("storagev2_number")
const bucketNumberInPlugin = scriptPlugin.createStorageNumber("storagev2_number")

// this test checks the validation of the max value size
runOnce("storagev2.ts", async () => {
    await testJson()
    await testNumber()
    sendScriptCompletion()
})

async function testJson() {
    // basic get set checks
    await bucketJson.set("1", {
        a: 10,
        b: "hello"
    })

    const entry = await bucketJson.get("1")
    assertExpected(10, entry?.value.a)
    assertExpected("hello", entry?.value.b)

    const count = await bucketJson.count()
    assertExpected(1, count)

    // ensure this didn't touch the plugin bucket
    const countInPlugin = await bucketJsonPlugin.count()
    assertExpected(0, countInPlugin)
    const entryInPlugin = await bucketJsonPlugin.get("1")
    assertExpected(undefined, entryInPlugin)
    let results = await bucketJsonPlugin.list({})
    assertExpected(0, results.length)

    // ensure that touching the plugin bucket does not touch the guild bucket
    await bucketJsonPlugin.set("1", {
        a: 20,
        b: "world"
    })

    const entryPlugin = await bucketJsonPlugin.get("1")
    assertExpected(20, entryPlugin?.value.a)
    assertExpected("world", entryPlugin?.value.b)

    const entryGuild = await bucketJson.get("1")
    assertExpected(10, entryGuild?.value.a)
    assertExpected("hello", entryGuild?.value.b)

    // check delete all
    await bucketJsonPlugin.deleteAll()
    assertExpected(await bucketJsonPlugin.count(), 0)
    assertExpected(await bucketJson.count(), 1)

    await bucketJsonPlugin.set("1", { a: 30, b: "!" })
    await bucketJson.deleteAll()
    assertExpected(await bucketJsonPlugin.count(), 1)
    assertExpected(await bucketJson.count(), 0)

    // reset
    await bucketJson.set("1", { a: 40, b: "wahoo" })

    // check single delete
    await bucketJsonPlugin.delete("1")

    assertExpected(await bucketJsonPlugin.count(), 0)
    assertExpected(await bucketJson.count(), 1)

    await bucketJsonPlugin.set("1", { a: 30, b: "!" })

    await bucketJson.delete("1")

    assertExpected(await bucketJsonPlugin.count(), 1)
    assertExpected(await bucketJson.count(), 0)

    // Complete!
}

async function testNumber() {
    await bucketNumber.incr("a", 1)
    const entry = await bucketNumber.get("a")
    assertExpected(1, entry?.value)

    const entryPlugin = await bucketNumberInPlugin.get("a")
    assertExpected(entryPlugin, undefined)

    await bucketNumber.incr("b", 3)
    await bucketNumber.incr("c", 2)

    const sorted = await bucketNumber.sortedList("Descending")
    assertExpected("b", sorted[0].key)
    assertExpected("c", sorted[1].key)
    assertExpected("a", sorted[2].key)

    const sortedPlugin = await bucketNumberInPlugin.sortedList("Descending")
    assertExpected(0, sortedPlugin.length)
}
