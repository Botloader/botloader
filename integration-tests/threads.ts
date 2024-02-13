import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const testForumId = "1206877897529364541"
const threadsTestChannelId = "1206878101473329183"

runOnce("threads.ts", async () => {
    await testPublicStandaloneThread()

    sendScriptCompletion()
})

async function testPublicStandaloneThread() {
    let channel = await Discord.createStandaloneThread({
        // invitable: true,
        channelId: threadsTestChannelId,
        kind: "PublicThread",
        name: "int-test-test-thread",
        autoArchiveDurationMinutes: 10080,
    })

    assertExpected("PublicThread", channel.kind)
    assertExpected(10080, channel.threadMetadata.autoArchiveDurationMinutes)
    assertExpected("int-test-test-thread", channel.name)

    channel = await channel.archive()
    assertExpected(true, channel.threadMetadata.archived)
}