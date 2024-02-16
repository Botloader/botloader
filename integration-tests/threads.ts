import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const testForumId = "1206877897529364541"
const testMemberId = "204255221017214977"
const threadsTestChannelId = "1206878101473329183"

runOnce("threads.ts", async () => {
    await testPublicStandaloneThread()
    await testPrivateStandaloneThread()
    await testMessageThread()
    await testForumThread()

    sendScriptCompletion()
})

async function testPublicStandaloneThread() {
    let channel = await Discord.createStandaloneThread({
        // invitable: true,
        channelId: threadsTestChannelId,
        kind: "PublicThread",
        name: "int-test-public-thread",
        autoArchiveDurationMinutes: 10080,
    })

    assertExpected("PublicThread", channel.kind)
    assertExpected(10080, channel.threadMetadata.autoArchiveDurationMinutes)
    assertExpected("int-test-public-thread", channel.name)

    // check if it appears in the active threads list
    const activeThreads = await Discord.getActiveThreads()
    const found = activeThreads.threads.find(v => v.id === channel.id)
    assertExpected(true, Boolean(found))

    channel = await channel.archive()
    assertExpected(true, channel.threadMetadata.archived)

    // test list archived threads
    const archivedThreads = await Discord.getPublicArchivedThreads({
        channelId: threadsTestChannelId,
    })

    const archivedThread = archivedThreads.threads.some(v => v.id === channel.id)
    assertExpected(true, archivedThread)
}

async function testPrivateStandaloneThread() {
    let channel = await Discord.createStandaloneThread({
        // invitable: true,
        channelId: threadsTestChannelId,
        kind: "PrivateThread",
        name: "int-test-private-thread",
        autoArchiveDurationMinutes: 10080,
    })

    assertExpected("PrivateThread", channel.kind)
    assertExpected(10080, channel.threadMetadata.autoArchiveDurationMinutes)
    assertExpected("int-test-private-thread", channel.name)

    // test guild list
    const activeThreads = await Discord.getActiveThreads()
    const foundThead = activeThreads.threads.find(v => v.id === channel.id)
    assertExpected(true, Boolean(foundThead))

    // test adding / removing members
    await Discord.addThreadMember(channel.id, testMemberId)

    let members = await Discord.getThreadMembers({
        channelId: channel.id,
    })
    const foundMember = members.find(v => v.id === channel.id && v.userId === testMemberId)
    assertExpected(true, Boolean(foundMember))

    // try removing
    await Discord.removeThreadMember(channel.id, testMemberId)

    const activeThreadsPost = await Discord.getActiveThreads()
    const foundMemberPost = activeThreadsPost.members.find(v => v.id === channel.id && v.userId === testMemberId)
    assertExpected(false, Boolean(foundMemberPost))

    // test archiving
    channel = await channel.archive()
    assertExpected(true, channel.threadMetadata.archived)

    // test list archived threads
    const archivedThreads = await Discord.getPrivateArchivedThreads({
        channelId: threadsTestChannelId,
    })

    const archivedThread = archivedThreads.threads.some(v => v.id === channel.id)
    assertExpected(true, archivedThread)
}

async function testMessageThread() {
    const msg = await Discord.createMessage(threadsTestChannelId, { content: "thread should be created on this message" })
    let channel = await msg.createThread({
        name: "int-test-message-thread",
        autoArchiveDurationMinutes: 10080
    })

    assertExpected("PublicThread", channel.kind)
    assertExpected(10080, channel.threadMetadata.autoArchiveDurationMinutes)
    assertExpected("int-test-message-thread", channel.name)

    // check if it appears in the active threads list
    const activeThreads = await Discord.getActiveThreads()
    const found = activeThreads.threads.find(v => v.id === channel.id)
    assertExpected(true, Boolean(found))

    channel = await channel.archive()
    assertExpected(true, channel.threadMetadata.archived)
}

async function testForumThread() {
    let { thread: channel, message } = await Discord.createForumThread({
        channelId: testForumId,
        message: { content: "test thread" },
        name: "int-test-forum-thread",
        autoArchiveDurationMinutes: 60,
    })

    assertExpected("PublicThread", channel.kind)
    assertExpected(60, channel.threadMetadata.autoArchiveDurationMinutes)
    assertExpected("int-test-forum-thread", channel.name)

    // check if it appears in the active threads list
    const activeThreads = await Discord.getActiveThreads()
    const found = activeThreads.threads.find(v => v.id === channel.id)
    assertExpected(true, Boolean(found))

    channel = await channel.archive()
    assertExpected(true, channel.threadMetadata.archived)
}