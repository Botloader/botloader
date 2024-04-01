import { assertExpected, runOnce, sendScriptCompletion } from "lib";
import { Discord } from 'botloader';

runOnce(script.name, async () => {
    let botUser = Discord.getBotUser();
    let guildPerms = await Discord.getMemberGuildPermissions(botUser.id);
    console.log("Guild perms: ", guildPerms.value.toString());
    console.log("Guild perms: ", guildPerms.toArray());

    let channelPerms = await Discord.getMemberChannelPermissions(botUser.id, "968927478775160933");
    console.log("Channel perms, guild: ", channelPerms.guild.toArray());
    console.log("Channel perms, channel: ", channelPerms.channel.toArray());

    assertExpected(guildPerms.value.toString(), channelPerms.guild.toString());

    sendScriptCompletion(script.name);
})