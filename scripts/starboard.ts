import { Discord } from 'botloader';

const STARBOARD_CHANNEL = "953675179958083625";
const STARBOARD_THRESHOLD = 3;

function isStarboardReaction(emoji: Discord.ReactionType) {
    if ("unicode" in emoji) {
        if (emoji.unicode === "⭐") {
            return true;
        }
    }

    return false;
}

script.on("MESSAGE_REACTION_ADD", async evt => {
    if (isStarboardReaction(evt.emoji)) {
        const message = await Discord.getMessage(evt.channelId, evt.messageId);
        const reaction = message.reactions.find(v => isStarboardReaction(v.emoji));
        if (reaction && reaction.count >= STARBOARD_THRESHOLD && !reaction.me && message.content.length > 0) {

            // to avoid posting the same message more than once we add our own reaction
            // and check if we reacted ourselves above
            await Discord.createReaction(evt.channelId, evt.messageId, {unicode: "⭐"})

            await Discord.createMessage(STARBOARD_CHANNEL, {
                allowedMentions: { parse: [] },
                embeds: [{
                    author: {
                        name: message.author.username + message.author.discriminator,
                        iconUrl: userAvatar(message.author),
                    },
                    description: message.content,
                }],
            })
        }
    }
})

// this function will get moved into btoloader eventually
function userAvatar(user: Discord.User) {
    const base = "https://cdn.discordapp.com/"
    if (user.avatar) {
        return base + `avatars/${user.id}/${user.avatar}.png?size=64`
    }

    const parsedDiscrim = parseInt(user.discriminator);
    return base + `embed/avatars/${parsedDiscrim % 5}.png?size=64`
}