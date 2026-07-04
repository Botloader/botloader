import { Commands, Discord } from "botloader";

const bannedWords = [
    "badword1", "badword2", "badword3", "badword4", "badword5",
    "badword6", "badword7", "badword8", "badword9", "badword10",
];

const warnings = script.createStorageJson<number>("mod_warnings");
const modLog = script.createStorageJson<{
    action: string,
    userId: string,
    moderatorId: string,
    reason: string,
    at: number,
}>("mod_log");

function containsBannedWord(content: string): string | null {
    const lowered = content.toLowerCase();
    for (const word of bannedWords) {
        if (lowered.includes(word)) {
            return word;
        }
    }
    return null;
}

script.on("MESSAGE_CREATE", async (msg) => {
    if (!msg.author || msg.author.bot) {
        return;
    }

    const hit = containsBannedWord(msg.content);
    if (hit) {
        await Discord.deleteMessage(msg.channelId, msg.id);
        const count = (await warnings.get(msg.author.id))?.value ?? 0;
        await warnings.set(msg.author.id, count + 1);
    }
});

script.createCommand(
    Commands.slashCommand("warn", "warn a user")
        .addOptionString("user_id", "user to warn")
        .addOptionString("reason", "reason for the warning", { required: false })
        .build(async (ctx, args) => {
            const count = (await warnings.get(args.user_id))?.value ?? 0;
            await warnings.set(args.user_id, count + 1);
            await modLog.set(`${Date.now()}`, {
                action: "warn",
                userId: args.user_id,
                moderatorId: ctx.member.user.id,
                reason: args.reason ?? "no reason given",
                at: Date.now(),
            });
            await ctx.sendResponse(`warned, they now have ${count + 1} warnings`);
        })
);

script.createCommand(
    Commands.slashCommand("warnings", "check how many warnings a user has")
        .addOptionString("user_id", "user to check")
        .build(async (ctx, args) => {
            const count = (await warnings.get(args.user_id))?.value ?? 0;
            await ctx.sendResponse(`${args.user_id} has ${count} warnings`);
        })
);

script.createCommand(
    Commands.slashCommand("clear-warnings", "reset a user's warnings")
        .addOptionString("user_id", "user to clear")
        .build(async (ctx, args) => {
            await warnings.del(args.user_id);
            await ctx.sendResponse("warnings cleared");
        })
);
