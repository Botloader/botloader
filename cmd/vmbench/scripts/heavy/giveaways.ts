import { Commands, Discord } from "botloader";

interface Giveaway {
    id: string,
    prize: string,
    hostId: string,
    channelId: string,
    messageId: string | null,
    winnerCount: number,
    entrants: string[],
    endsAt: number,
    ended: boolean,
    winners: string[],
    requiredRoleId: string | null,
}

const giveaways = script.createStorageJson<Giveaway>("giveaways");
const wins = script.createStorageNumber("giveaway_wins");

const DURATION_PRESETS: Record<string, number> = {
    "1h": 3_600_000,
    "6h": 6 * 3_600_000,
    "12h": 12 * 3_600_000,
    "1d": 24 * 3_600_000,
    "3d": 3 * 24 * 3_600_000,
    "1w": 7 * 24 * 3_600_000,
};

function parseDuration(input: string): number | null {
    if (DURATION_PRESETS[input]) {
        return DURATION_PRESETS[input];
    }
    const match = /^(\d+)(m|h|d)$/.exec(input);
    if (!match) {
        return null;
    }
    const amount = parseInt(match[1], 10);
    switch (match[2]) {
        case "m": return amount * 60_000;
        case "h": return amount * 3_600_000;
        case "d": return amount * 24 * 3_600_000;
        default: return null;
    }
}

function pickWinners(entrants: string[], count: number): string[] {
    const pool = [...entrants];
    const winners: string[] = [];
    while (winners.length < count && pool.length > 0) {
        const index = Math.floor(Math.random() * pool.length);
        winners.push(pool.splice(index, 1)[0]);
    }
    return winners;
}

function renderGiveaway(giveaway: Giveaway): string {
    const lines = [
        `🎉 **GIVEAWAY: ${giveaway.prize}** 🎉`,
        `hosted by <@${giveaway.hostId}>`,
        `${giveaway.winnerCount} winner(s), ${giveaway.entrants.length} entries`,
    ];
    if (giveaway.requiredRoleId) {
        lines.push(`requires <@&${giveaway.requiredRoleId}>`);
    }
    if (giveaway.ended) {
        lines.push(
            giveaway.winners.length > 0
                ? `winners: ${giveaway.winners.map((w) => `<@${w}>`).join(", ")}`
                : "no valid entries, no winners"
        );
    } else {
        const remaining = Math.max(0, giveaway.endsAt - Date.now());
        lines.push(`ends in ${Math.ceil(remaining / 60_000)} minutes - react with 🎉 to enter`);
    }
    return lines.join("\n");
}

async function endGiveaway(entry: { key: string, value: Giveaway }): Promise<void> {
    const winners = pickWinners(entry.value.entrants, entry.value.winnerCount);
    const ended: Giveaway = { ...entry.value, ended: true, winners };
    await giveaways.set(entry.key, ended);

    for (const winner of winners) {
        await wins.incr(winner, 1);
    }

    if (ended.messageId) {
        await Discord.editMessage(ended.channelId, ended.messageId, {
            content: renderGiveaway(ended),
        });
    }
    if (winners.length > 0) {
        await Discord.createMessage(ended.channelId, {
            content: `congratulations ${winners.map((w) => `<@${w}>`).join(", ")}, you won **${ended.prize}**!`,
        });
    }
}

script.on("MESSAGE_REACTION_ADD", async (evt) => {
    if (evt.emoji.name !== "🎉") {
        return;
    }

    const all = await giveaways.list({ limit: 50 });
    const giveaway = all.find((e) => e.value.messageId === evt.messageId && !e.value.ended);
    if (!giveaway) {
        return;
    }

    if (giveaway.value.entrants.includes(evt.userId)) {
        return;
    }

    if (giveaway.value.requiredRoleId && evt.member) {
        if (!evt.member.roles.includes(giveaway.value.requiredRoleId)) {
            return;
        }
    }

    await giveaways.set(giveaway.key, {
        ...giveaway.value,
        entrants: [...giveaway.value.entrants, evt.userId],
    });
});

script.createCommand(
    Commands.slashCommand("giveaway-start", "start a giveaway")
        .addOptionString("prize", "what you're giving away")
        .addOptionString("duration", "e.g. 30m, 6h, 1d")
        .addOptionInteger("winners", "number of winners", { required: false })
        .addOptionString("required_role_id", "role required to enter", { required: false })
        .build(async (ctx, args) => {
            const durationMs = parseDuration(args.duration);
            if (!durationMs) {
                await ctx.sendResponse("invalid duration, use something like 30m, 6h or 1d");
                return;
            }

            const giveaway: Giveaway = {
                id: `${Date.now()}`,
                prize: args.prize,
                hostId: ctx.member.user.id,
                channelId: ctx.channelId,
                messageId: null,
                winnerCount: Math.max(1, args.winners ?? 1),
                entrants: [],
                endsAt: Date.now() + durationMs,
                ended: false,
                winners: [],
                requiredRoleId: args.required_role_id ?? null,
            };

            await giveaways.set(giveaway.id, giveaway);
            await ctx.sendResponse(renderGiveaway(giveaway));
        })
);

script.createCommand(
    Commands.slashCommand("giveaway-end", "end a giveaway early")
        .addOptionString("id", "giveaway id")
        .build(async (ctx, args) => {
            const entry = await giveaways.get(args.id);
            if (!entry || entry.value.ended) {
                await ctx.sendResponse("no running giveaway with that id");
                return;
            }
            await endGiveaway({ key: args.id, value: entry.value });
            await ctx.sendResponse("giveaway ended");
        })
);

script.createCommand(
    Commands.slashCommand("giveaway-reroll", "pick new winners for an ended giveaway")
        .addOptionString("id", "giveaway id")
        .build(async (ctx, args) => {
            const entry = await giveaways.get(args.id);
            if (!entry || !entry.value.ended) {
                await ctx.sendResponse("no ended giveaway with that id");
                return;
            }
            const winners = pickWinners(entry.value.entrants, entry.value.winnerCount);
            await giveaways.set(args.id, { ...entry.value, winners });
            await ctx.sendResponse(
                winners.length > 0
                    ? `new winners: ${winners.map((w) => `<@${w}>`).join(", ")}`
                    : "no entrants to pick from"
            );
        })
);

script.createCommand(
    Commands.slashCommand("giveaway-wins", "see who has won the most giveaways")
        .build(async (ctx) => {
            const top = await wins.sortedList("Descending", { limit: 10 });
            const lines = top.map((e, i) => `${i + 1}. <@${e.key}>: ${e.value} wins`);
            await ctx.sendResponse(lines.join("\n") || "nobody has won anything yet");
        })
);

script.onInterval("giveaway_ticker", 1, async () => {
    const all = await giveaways.list({ limit: 50 });
    for (const entry of all) {
        if (!entry.value.ended && entry.value.endsAt <= Date.now()) {
            await endGiveaway(entry);
        }
    }
});
