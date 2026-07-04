import { Commands, Discord } from "botloader";

const welcomeMessages = [
    "Welcome to the server, {user}!",
    "Glad to have you here, {user}!",
    "{user} just joined, say hi!",
    "A wild {user} appeared!",
];

const farewellMessages = [
    "{user} has left the building.",
    "Goodbye {user}, we'll miss you.",
];

const settingsVar = script.createStorageVarJson<{
    channelId: string | null,
    enabled: boolean,
}>("welcome_settings");

function pickMessage(pool: string[], username: string): string {
    const template = pool[Math.floor(Math.random() * pool.length)];
    return template.replace("{user}", username);
}

script.on("MEMBER_ADD", async (member) => {
    const settings = await settingsVar.get();
    if (!settings || !settings.value.enabled || !settings.value.channelId) {
        return;
    }

    await Discord.createMessage(settings.value.channelId, {
        content: pickMessage(welcomeMessages, member.user.username),
    });
});

script.on("MEMBER_REMOVE", async (evt) => {
    const settings = await settingsVar.get();
    if (!settings || !settings.value.enabled || !settings.value.channelId) {
        return;
    }

    await Discord.createMessage(settings.value.channelId, {
        content: pickMessage(farewellMessages, evt.user.username),
    });
});

script.createCommand(
    Commands.slashCommand("welcome-channel", "set the welcome channel")
        .addOptionString("channel_id", "channel to post welcome messages in")
        .build(async (ctx, args) => {
            await settingsVar.set({ channelId: args.channel_id, enabled: true });
            await ctx.sendResponse(`welcome channel set to ${args.channel_id}`);
        })
);

script.createCommand(
    Commands.slashCommand("welcome-toggle", "enable or disable welcome messages")
        .addOptionBoolean("enabled", "whether welcome messages are on")
        .build(async (ctx, args) => {
            const current = await settingsVar.get();
            await settingsVar.set({
                channelId: current?.value.channelId ?? null,
                enabled: args.enabled,
            });
            await ctx.sendResponse(`welcome messages ${args.enabled ? "enabled" : "disabled"}`);
        })
);
