import { Commands, Discord } from "botloader";

interface Poll {
    question: string,
    options: string[],
    votes: Record<string, number>,
    voters: string[],
    creatorId: string,
    channelId: string,
    messageId: string | null,
    endsAt: number,
    closed: boolean,
}

const polls = script.createStorageJson<Poll>("polls");

const NUMBER_EMOJIS = ["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟"];

function renderPoll(poll: Poll): string {
    const total = Object.values(poll.votes).reduce((a, b) => a + b, 0);
    const lines = [`**${poll.question}**`, ""];

    poll.options.forEach((option, i) => {
        const count = poll.votes[`${i}`] ?? 0;
        const pct = total === 0 ? 0 : Math.round((count / total) * 100);
        const bar = "█".repeat(Math.round(pct / 10)).padEnd(10, "░");
        lines.push(`${NUMBER_EMOJIS[i]} ${option}`);
        lines.push(`   ${bar} ${count} votes (${pct}%)`);
    });

    if (poll.closed) {
        lines.push("", "*this poll has ended*");
    }

    return lines.join("\n");
}

script.createCommand(
    Commands.slashCommand("poll", "create a poll")
        .addOptionString("question", "what to ask")
        .addOptionString("options", "comma separated options (max 10)")
        .addOptionInteger("duration_minutes", "how long the poll runs", { required: false })
        .build(async (ctx, args) => {
            const options = args.options.split(",").map((v) => v.trim()).filter((v) => v.length > 0);
            if (options.length < 2 || options.length > 10) {
                await ctx.sendResponse("polls need between 2 and 10 options");
                return;
            }

            const poll: Poll = {
                question: args.question,
                options,
                votes: {},
                voters: [],
                creatorId: ctx.member.user.id,
                channelId: ctx.channelId,
                messageId: null,
                endsAt: Date.now() + (args.duration_minutes ?? 60) * 60_000,
                closed: false,
            };

            const id = `${Date.now()}_${ctx.member.user.id}`;
            await polls.set(id, poll);
            await ctx.sendResponse(renderPoll(poll));
        })
);

script.onInterval("close_expired_polls", 5, async () => {
    const open = await polls.list({ limit: 100 });
    for (const entry of open) {
        if (!entry.value.closed && entry.value.endsAt < Date.now()) {
            const closed = { ...entry.value, closed: true };
            await polls.set(entry.key, closed);
            if (closed.messageId) {
                await Discord.editMessage(closed.channelId, closed.messageId, {
                    content: renderPoll(closed),
                });
            }
        }
    }
});
