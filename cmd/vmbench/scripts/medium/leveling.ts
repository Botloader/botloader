import { Commands } from "botloader";

const xpStore = script.createStorageNumber("leveling_xp");
const cooldowns = new Map<string, number>();

const levelThresholds: number[] = [];
for (let i = 0; i < 100; i++) {
    levelThresholds.push(Math.floor(100 * Math.pow(1.2, i)));
}

function levelForXp(xp: number): number {
    let level = 0;
    let total = 0;
    for (const threshold of levelThresholds) {
        total += threshold;
        if (xp < total) {
            break;
        }
        level++;
    }
    return level;
}

function xpForNextLevel(xp: number): number {
    let total = 0;
    for (const threshold of levelThresholds) {
        total += threshold;
        if (xp < total) {
            return total - xp;
        }
    }
    return 0;
}

script.on("MESSAGE_CREATE", async (msg) => {
    if (!msg.author || msg.author.bot) {
        return;
    }

    const last = cooldowns.get(msg.author.id) ?? 0;
    if (Date.now() - last < 60_000) {
        return;
    }
    cooldowns.set(msg.author.id, Date.now());

    const gained = 15 + Math.floor(Math.random() * 10);
    await xpStore.incr(msg.author.id, gained);
});

script.createCommand(
    Commands.slashCommand("rank", "check your level and xp")
        .build(async (ctx) => {
            const entry = await xpStore.get(ctx.member.user.id);
            const xp = entry?.value ?? 0;
            const level = levelForXp(xp);
            await ctx.sendResponse(
                `you are level ${level} with ${xp} xp, ${xpForNextLevel(xp)} xp until next level`
            );
        })
);

script.createCommand(
    Commands.slashCommand("leaderboard", "show the top users by xp")
        .addOptionInteger("limit", "how many entries to show", { required: false })
        .build(async (ctx, args) => {
            const entries = await xpStore.sortedList("Descending", {
                limit: args.limit ?? 10,
            });
            const lines = entries.map(
                (e, i) => `${i + 1}. <@${e.key}> - level ${levelForXp(e.value)} (${e.value} xp)`
            );
            await ctx.sendResponse(lines.join("\n") || "no entries yet");
        })
);

script.onInterval("decay_cooldowns", 10, () => {
    const cutoff = Date.now() - 300_000;
    for (const [key, at] of cooldowns) {
        if (at < cutoff) {
            cooldowns.delete(key);
        }
    }
});
