import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const testForumId = "1206877897529364541"
const testMemberId = "204255221017214977"
const threadsTestChannelId = "1206878101473329183"

const threadStorageState = script.createStorageVarJson<"adding_member" | "removing_member" | "archiving" | "complete">("threads_events_state")
const threadStorage = script.createStorageVarJson<string>("threads_events")

runOnce("threads_events.ts", async () => {
    let channel = await Discord.createStandaloneThread({
        // invitable: true,
        channelId: threadsTestChannelId,
        kind: "PrivateThread",
        name: "int-test-thread-events",
        autoArchiveDurationMinutes: 10080,
    })
})

script.on("THREAD_CREATE", async (evt) => {
    if (evt.name === "int-test-thread-events") {
        // successfully created
        await threadStorage.set(evt.id)
        await threadStorageState.set("adding_member")
        await Discord.addThreadMember(evt.id, testMemberId)
    }
})

script.on("THREAD_MEMBERS_UPDATE", async (evt) => {
    const state = await threadStorageState.get()
    if (evt.id === (await threadStorage.get())?.value) {
        if (state?.value === "adding_member") {
            if (evt.addedMembers.some(v => testMemberId === v.userId)) {
                // success
                await threadStorageState.set("removing_member")
                await Discord.removeThreadMember(evt.id, testMemberId)
            }
        } else if (state?.value === "removing_member") {
            if (evt.removedMemberIds.includes(testMemberId)) {
                // success
                await threadStorageState.set("archiving")
                await Discord.editThread({
                    channelId: evt.id,
                    archived: true,
                })
            }
        }
    }
})

script.on("THREAD_UPDATE", async (evt) => {
    const state = await threadStorageState.get()
    if (evt.id === (await threadStorage.get())?.value) {
        if (state?.value === "archiving") {
            if (evt.threadMetadata.archived) {
                // success
                const newState = await threadStorageState.delete()
                if (newState) {
                    sendScriptCompletion()
                }
            }
        }
    }
})