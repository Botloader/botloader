import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

// Red dot taken from wikipedia
const RED_DOT = `data:image/png;base64,iVBORw0KGgoAAA
ANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4
//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU
5ErkJggg==`

runOnce(script.name, async () => {
    const emoji = await Discord.createEmoji({
        data: RED_DOT,
        name: "IG_TEST",
    })

    assertExpected("IG_TEST", emoji.name)
    assertExpected(null, emoji.roles)
    assertExpected(false, emoji.animated)

    const edited = await Discord.editEmoji(emoji.id, {
        name: "IG_TEST_2",
        roles: ["941079186087481344"]
    })


    assertExpected("IG_TEST_2", edited.name)
    assertExpected(1, edited.roles?.length)
    assertExpected("941079186087481344", edited.roles![0])
    assertExpected(false, edited.animated)

    await Discord.createMessage("531120790318350338", {
        content: `emoji: ` + edited.format()
    })

    await deleter.schedule({
        data: edited.id,
        executeAt: new Date(Date.now() + 1000)
    })
})

const deleter = script.createTaskBucket<string>({
    name: "emoji"
}, async (task) => {

    const foundEmoji = await Discord.getEmoji(task.data)
    assertExpected(true, foundEmoji !== null)

    await Discord.deleteEmoji(task.data)

    verifyDeleter.schedule({
        data: task.data,
        executeAt: new Date(Date.now() + 5000)
    })
})

const verifyDeleter = script.createTaskBucket<string>({
    name: "emoji_verify"
}, async (task) => {
    const foundEmoji = await Discord.getEmoji(task.data)
    assertExpected(true, foundEmoji === undefined)

    sendScriptCompletion(script.name)
})