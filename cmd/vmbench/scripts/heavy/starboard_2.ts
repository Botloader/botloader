import { Commands, Discord } from "botloader";

interface StarboardEntry {
    originalMessageId: string,
    starboardMessageId: string | null,
    channelId: string,
    authorId: string,
    stars: number,
    content: string,
}

interface StarboardConfig {
    channelId: string | null,
    threshold: number,
    selfStarAllowed: boolean,
    emoji: string,
}

const DEFAULT_CONFIG: StarboardConfig = {
    channelId: null,
    threshold: 3,
    selfStarAllowed: false,
    emoji: "⭐",
};

const entries = script.createStorageJson<StarboardEntry>("starboard_entries");
const configVar = script.createStorageVarJson<StarboardConfig>("starboard_config");
const starGivers = script.createStorageNumber("starboard_givers");
const starReceivers = script.createStorageNumber("starboard_receivers");

const MEDALS = ["🥇", "🥈", "🥉"];

function renderStarboardMessage(entry: StarboardEntry, emoji: string): string {
    return [
        `${emoji} **${entry.stars}** in <#${entry.channelId}>`,
        `by <@${entry.authorId}>`,
        "",
        entry.content.length > 500 ? entry.content.slice(0, 500) + "…" : entry.content,
    ].join("\n");
}

async function getConfig(): Promise<StarboardConfig> {
    return (await configVar.get())?.value ?? DEFAULT_CONFIG;
}

script.on("MESSAGE_REACTION_ADD", async (evt) => {
    const config = await getConfig();
    if (!config.channelId || evt.emoji.name !== config.emoji) {
        return;
    }

    const existing = await entries.get(evt.messageId);
    if (existing) {
        const updated = { ...existing.value, stars: existing.value.stars + 1 };
        await entries.set(evt.messageId, updated);
        await starGivers.incr(evt.userId, 1);
        await starReceivers.incr(updated.authorId, 1);

        if (updated.starboardMessageId) {
            await Discord.editMessage(config.channelId, updated.starboardMessageId, {
                content: renderStarboardMessage(updated, config.emoji),
            });
        } else if (updated.stars >= config.threshold) {
            const posted = await Discord.createMessage(config.channelId, {
                content: renderStarboardMessage(updated, config.emoji),
            });
            await entries.set(evt.messageId, { ...updated, starboardMessageId: posted.id });
        }
        return;
    }

    const message = await Discord.getMessage(evt.channelId, evt.messageId);
    if (!config.selfStarAllowed && message.author.id === evt.userId) {
        return;
    }

    await entries.set(evt.messageId, {
        originalMessageId: evt.messageId,
        starboardMessageId: null,
        channelId: evt.channelId,
        authorId: message.author.id,
        stars: 1,
        content: message.content,
    });
});

script.on("MESSAGE_REACTION_REMOVE", async (evt) => {
    const config = await getConfig();
    if (evt.emoji.name !== config.emoji) {
        return;
    }

    const existing = await entries.get(evt.messageId);
    if (!existing) {
        return;
    }

    const updated = { ...existing.value, stars: Math.max(0, existing.value.stars - 1) };
    await entries.set(evt.messageId, updated);

    if (updated.starboardMessageId && config.channelId) {
        if (updated.stars < config.threshold) {
            await Discord.deleteMessage(config.channelId, updated.starboardMessageId);
            await entries.set(evt.messageId, { ...updated, starboardMessageId: null });
        } else {
            await Discord.editMessage(config.channelId, updated.starboardMessageId, {
                content: renderStarboardMessage(updated, config.emoji),
            });
        }
    }
});

script.createCommand(
    Commands.slashCommand("v2-starboard-channel", "set the starboard channel")
        .addOptionString("channel_id", "channel to post starred messages in")
        .build(async (ctx, args) => {
            const config = await getConfig();
            await configVar.set({ ...config, channelId: args.channel_id });
            await ctx.sendResponse(`starboard channel set to <#${args.channel_id}>`);
        })
);

script.createCommand(
    Commands.slashCommand("v2-starboard-threshold", "set how many stars are needed")
        .addOptionInteger("threshold", "minimum star count")
        .build(async (ctx, args) => {
            if (args.threshold < 1 || args.threshold > 100) {
                await ctx.sendResponse("threshold must be between 1 and 100");
                return;
            }
            const config = await getConfig();
            await configVar.set({ ...config, threshold: args.threshold });
            await ctx.sendResponse(`starboard threshold set to ${args.threshold}`);
        })
);

script.createCommand(
    Commands.slashCommand("v2-star-stats", "show top star givers and receivers")
        .build(async (ctx) => {
            const givers = await starGivers.sortedList("Descending", { limit: 3 });
            const receivers = await starReceivers.sortedList("Descending", { limit: 3 });

            const lines = ["**Top star receivers**"];
            receivers.forEach((e, i) => lines.push(`${MEDALS[i] ?? "•"} <@${e.key}>: ${e.value}`));
            lines.push("", "**Top star givers**");
            givers.forEach((e, i) => lines.push(`${MEDALS[i] ?? "•"} <@${e.key}>: ${e.value}`));

            await ctx.sendResponse(lines.join("\n"));
        })
);

script.createCommand(
    Commands.messageCommand("v2 Star Info")
        .build(async (ctx, target) => {
            const entry = await entries.get(target.id);
            if (!entry) {
                await ctx.sendResponse("that message has no stars");
                return;
            }
            await ctx.sendResponse(`that message has ${entry.value.stars} stars`);
        })
);
