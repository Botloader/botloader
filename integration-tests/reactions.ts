import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const emoji = { unicode: "😀" }
const channelId = "531120790318350338";

runOnce(script.name, async () => {
    let msg = await Discord.createMessage(channelId, { content: "Reactions testing 1" });
    const messageId = msg.id;


    let reactions = await Discord.getReactions(channelId, messageId, emoji);
    assertExpected(0, reactions.length);

    // try creating
    await Discord.createReaction(channelId, messageId, emoji);
    reactions = await Discord.getReactions(channelId, messageId, emoji);
    assertExpected(1, reactions.length);

    // try deleting
    await Discord.deleteOwnReaction(channelId, messageId, emoji);
    reactions = await Discord.getReactions(channelId, messageId, emoji);
    assertExpected(0, reactions.length);

    // try deleting all
    await Discord.createReaction(channelId, messageId, emoji);
    reactions = await Discord.getReactions(channelId, messageId, emoji);
    assertExpected(1, reactions.length);
    await Discord.deleteAllReactions(channelId, messageId);
    reactions = await Discord.getReactions(channelId, messageId, emoji);
    assertExpected(0, reactions.length);

    // try deleting all by emoji
    await Discord.createReaction(channelId, messageId, emoji);
    reactions = await Discord.getReactions(channelId, messageId, emoji);
    assertExpected(1, reactions.length);
    await Discord.deleteAllEmojiReactions(channelId, messageId, emoji);
    reactions = await Discord.getReactions(channelId, messageId, emoji);
    assertExpected(0, reactions.length);

    sendScriptCompletion(script.name);
})