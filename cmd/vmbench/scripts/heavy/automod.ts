import { Commands, Discord } from "botloader";

interface AutomodRule {
    name: string,
    enabled: boolean,
    action: "delete" | "warn" | "mute",
    threshold: number,
}

const DEFAULT_RULES: Record<string, AutomodRule> = {
    caps: { name: "excessive caps", enabled: true, action: "delete", threshold: 70 },
    mentions: { name: "mass mentions", enabled: true, action: "warn", threshold: 5 },
    emoji: { name: "emoji spam", enabled: true, action: "delete", threshold: 10 },
    newlines: { name: "wall of text", enabled: false, action: "delete", threshold: 15 },
    repeats: { name: "repeated messages", enabled: true, action: "mute", threshold: 3 },
    invites: { name: "discord invites", enabled: true, action: "delete", threshold: 1 },
    links: { name: "links", enabled: false, action: "delete", threshold: 1 },
    zalgo: { name: "zalgo text", enabled: true, action: "delete", threshold: 1 },
};

const INVITE_PATTERN = /discord(?:\.gg|(?:app)?\.com\/invite)\/[a-zA-Z0-9-]+/;
const LINK_PATTERN = /https?:\/\/\S+/g;
const ZALGO_PATTERN = /[̀-ͯ҉]{3,}/;
const EMOJI_PATTERN = /<a?:\w+:\d+>|[\u{1F300}-\u{1FAFF}]/gu;

const rulesVar = script.createStorageVarJson<Record<string, AutomodRule>>("automod_rules");
const violations = script.createStorageNumber("automod_violations");
const recentMessages = new Map<string, { content: string, count: number, at: number }>();

function capsPercentage(content: string): number {
    const letters = content.replace(/[^a-zA-Z]/g, "");
    if (letters.length < 10) {
        return 0;
    }
    const caps = letters.replace(/[^A-Z]/g, "").length;
    return (caps / letters.length) * 100;
}

function countEmojis(content: string): number {
    const matches = content.match(EMOJI_PATTERN);
    return matches?.length ?? 0;
}

function checkMessage(content: string, mentionCount: number, rules: Record<string, AutomodRule>): AutomodRule | null {
    const checks: [string, boolean][] = [
        ["caps", capsPercentage(content) > (rules.caps?.threshold ?? 100)],
        ["mentions", mentionCount >= (rules.mentions?.threshold ?? 99)],
        ["emoji", countEmojis(content) >= (rules.emoji?.threshold ?? 99)],
        ["newlines", content.split("\n").length >= (rules.newlines?.threshold ?? 99)],
        ["invites", INVITE_PATTERN.test(content)],
        ["links", (content.match(LINK_PATTERN)?.length ?? 0) >= (rules.links?.threshold ?? 99)],
        ["zalgo", ZALGO_PATTERN.test(content)],
    ];

    for (const [key, hit] of checks) {
        const rule = rules[key];
        if (rule && rule.enabled && hit) {
            return rule;
        }
    }
    return null;
}

script.on("MESSAGE_CREATE", async (msg) => {
    if (!msg.author || msg.author.bot) {
        return;
    }

    const rules = (await rulesVar.get())?.value ?? DEFAULT_RULES;

    const previous = recentMessages.get(msg.author.id);
    if (previous && previous.content === msg.content && Date.now() - previous.at < 30_000) {
        previous.count += 1;
        previous.at = Date.now();
        const repeatRule = rules.repeats;
        if (repeatRule?.enabled && previous.count >= repeatRule.threshold) {
            await Discord.deleteMessage(msg.channelId, msg.id);
            await violations.incr(msg.author.id, 1);
            return;
        }
    } else {
        recentMessages.set(msg.author.id, { content: msg.content, count: 1, at: Date.now() });
    }

    const violated = checkMessage(msg.content, msg.mentions.length, rules);
    if (!violated) {
        return;
    }

    await violations.incr(msg.author.id, 1);
    if (violated.action === "delete" || violated.action === "mute") {
        await Discord.deleteMessage(msg.channelId, msg.id);
    }
    console.log(`automod: ${msg.author.id} violated rule '${violated.name}'`);
});

script.createCommand(
    Commands.slashCommand("automod-list", "list automod rules and their status")
        .build(async (ctx) => {
            const rules = (await rulesVar.get())?.value ?? DEFAULT_RULES;
            const lines = Object.entries(rules).map(
                ([key, rule]) =>
                    `${rule.enabled ? "🟢" : "🔴"} **${key}** (${rule.name}): ${rule.action} at threshold ${rule.threshold}`
            );
            await ctx.sendResponse(lines.join("\n"));
        })
);

script.createCommand(
    Commands.slashCommand("automod-toggle", "enable or disable an automod rule")
        .addOptionString("rule", "rule key, see /automod-list")
        .addOptionBoolean("enabled", "whether the rule is active")
        .build(async (ctx, args) => {
            const rules = (await rulesVar.get())?.value ?? { ...DEFAULT_RULES };
            const rule = rules[args.rule];
            if (!rule) {
                await ctx.sendResponse("unknown rule");
                return;
            }
            rule.enabled = args.enabled;
            await rulesVar.set(rules);
            await ctx.sendResponse(`rule '${args.rule}' ${args.enabled ? "enabled" : "disabled"}`);
        })
);

script.createCommand(
    Commands.slashCommand("automod-threshold", "change a rule's threshold")
        .addOptionString("rule", "rule key, see /automod-list")
        .addOptionInteger("threshold", "new threshold value")
        .build(async (ctx, args) => {
            const rules = (await rulesVar.get())?.value ?? { ...DEFAULT_RULES };
            const rule = rules[args.rule];
            if (!rule) {
                await ctx.sendResponse("unknown rule");
                return;
            }
            rule.threshold = args.threshold;
            await rulesVar.set(rules);
            await ctx.sendResponse(`rule '${args.rule}' threshold set to ${args.threshold}`);
        })
);

script.createCommand(
    Commands.slashCommand("automod-violations", "check a user's violation count")
        .addOptionString("user_id", "user to check")
        .build(async (ctx, args) => {
            const count = (await violations.get(args.user_id))?.value ?? 0;
            await ctx.sendResponse(`<@${args.user_id}> has ${count} automod violations`);
        })
);

script.onInterval("automod_cleanup", 15, () => {
    const cutoff = Date.now() - 120_000;
    for (const [key, entry] of recentMessages) {
        if (entry.at < cutoff) {
            recentMessages.delete(key);
        }
    }
});
