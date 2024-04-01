import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const channelId = "531120790318350338"

const storage = script.createStorageJson<string>("invites.ts")

script.on("INVITE_CREATE", async (evt) => {
    assertExpected(10, evt.maxUses)
    if (await storage.get("deleting")) {
        return
    }

    await storage.set("deleting", evt.code)

    // test get inivte
    const retrieved = await Discord.getInvite(evt.code)
    assertExpected(evt.code, retrieved.code)

    // test guild invites
    const guildInivtes = await Discord.getGuildInvites()
    assertExpected(true, Boolean(guildInivtes.find(v => v.code === evt.code)))

    // test channel invites
    const channelInvites = await Discord.getChannelInvites(channelId)
    assertExpected(true, Boolean(channelInvites.find(v => v.code === evt.code)))

    // test deleting invites
    await Discord.deleteInvite(evt.code)
})

script.on("INVITE_DELETE", async (evt) => {
    let deleting = await storage.get("deleting")
    if (!deleting) {
        return
    }

    if (deleting.value !== evt.code) {
        return
    }

    sendScriptCompletion(script.name)

})

runOnce(script.name, async () => {
    await Discord.createChannelInvite(channelId, {
        maxUses: 10,
        maxAgeSeconds: 200,
    })
})
