import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

runOnce("channels.ts", async () => {
    let bot = Discord.getBotUser();
    let channel = await Discord.createChannel({
        name: "gaming",
        topic: "We are gaming",
        permissionOverwrites: [
            // diasllow send messages for everyone
            Discord.PermissionOverwrite.everyone(new Discord.Permissions(), Discord.Permissions.SendMessages),
        ]
    });

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

    await Discord.deleteChannelPermission(channel.id, "Member", bot.id);
    await Discord.deleteChannel(channel.id);

    sendScriptCompletion();
})