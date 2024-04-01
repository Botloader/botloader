import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const bot = Discord.getBotUser();

script.on("CHANNEL_CREATE", async (channel) => {
    if (channel.name === "bl-chtest-1") {
        // give ourselves send messages
        await Discord.editChannelPermission(channel.id,
            Discord.PermissionOverwrite.member(bot.id, Discord.Permissions.SendMessages,
                new Discord.Permissions())
        );

        // put this channel under a category
        let edited = await Discord.editChannel(channel.id, {
            parentId: "531120790318350337",
        });

        if ("permissionOverwrites" in edited) {
            assertExpected(2, edited.permissionOverwrites.length);
        } else {
            throw new Error("No permission overwrites...");
        }
    } else if (channel.name === "bl-chtest-2") {
        assertExpected("Voice", channel.kind)
        await Discord.deleteChannel(channel.id)
    }
})

script.on("CHANNEL_UPDATE", async (channel) => {
    if (channel.name === "bl-chtest-1" && "parentId" in channel && channel.parentId === "531120790318350337") {
        if ("permissionOverwrites" in channel) {
            if (channel.permissionOverwrites.length === 2) {
                await Discord.deleteChannelPermission(channel.id, "Member", bot.id);
            } else if (channel.permissionOverwrites.length === 1) {
                await Discord.deleteChannel(channel.id);
            } else {
                throw new Error("unknown number of overwrites!");
            }
        } else {
            throw new Error("No permission overwrites...");
        }
    }
})

script.on("CHANNEL_DELETE", async (channel) => {
    if (channel.name === "bl-chtest-1") {
        await Discord.createChannel({
            name: "bl-chtest-2",
            kind: "Voice",
            permissionOverwrites: [
                // disallow send messages for everyone
                Discord.PermissionOverwrite.everyone(new Discord.Permissions(), Discord.Permissions.SendMessages),
            ]
        })
    } else if (channel.name === "bl-chtest-2") {
        sendScriptCompletion(script.name);
    }
})

runOnce(script.name, async () => {
    let channel = await Discord.createChannel({
        name: "bl-chtest-1",
        topic: "We are gaming",
        permissionOverwrites: [
            // disallow send messages for everyone
            Discord.PermissionOverwrite.everyone(new Discord.Permissions(), Discord.Permissions.SendMessages),
        ]
    });
})
