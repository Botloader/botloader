import { Commands, Discord } from "botloader";

interface RoleBinding {
    messageId: string,
    emoji: string,
    roleId: string,
}

const bindings = script.createStorageJson<RoleBinding>("reaction_role_bindings");

function bindingKey(messageId: string, emoji: string): string {
    return `${messageId}:${emoji}`;
}

script.on("MESSAGE_REACTION_ADD", async (evt) => {
    if (!evt.member || evt.member.user.bot) {
        return;
    }

    const emojiName = evt.emoji.name ?? "";
    const binding = await bindings.get(bindingKey(evt.messageId, emojiName));
    if (!binding) {
        return;
    }

    await Discord.addMemberRole(evt.userId, binding.value.roleId);
});

script.on("MESSAGE_REACTION_REMOVE", async (evt) => {
    const emojiName = evt.emoji.name ?? "";
    const binding = await bindings.get(bindingKey(evt.messageId, emojiName));
    if (!binding) {
        return;
    }

    await Discord.removeMemberRole(evt.userId, binding.value.roleId);
});

script.createCommand(
    Commands.slashCommand("reaction-role-add", "bind an emoji on a message to a role")
        .addOptionString("message_id", "message to watch")
        .addOptionString("emoji", "emoji that grants the role")
        .addOptionString("role_id", "role to grant")
        .build(async (ctx, args) => {
            await bindings.set(bindingKey(args.message_id, args.emoji), {
                messageId: args.message_id,
                emoji: args.emoji,
                roleId: args.role_id,
            });
            await ctx.sendResponse(`bound ${args.emoji} on ${args.message_id} to <@&${args.role_id}>`);
        })
);

script.createCommand(
    Commands.slashCommand("reaction-role-remove", "remove a reaction role binding")
        .addOptionString("message_id", "message the binding is on")
        .addOptionString("emoji", "emoji of the binding")
        .build(async (ctx, args) => {
            await bindings.del(bindingKey(args.message_id, args.emoji));
            await ctx.sendResponse("binding removed");
        })
);

script.createCommand(
    Commands.slashCommand("reaction-role-list", "list all reaction role bindings")
        .build(async (ctx) => {
            const all = await bindings.list({ limit: 50 });
            if (all.length === 0) {
                await ctx.sendResponse("no bindings configured");
                return;
            }
            const lines = all.map(
                (e) => `${e.value.emoji} on ${e.value.messageId} -> <@&${e.value.roleId}>`
            );
            await ctx.sendResponse(lines.join("\n"));
        })
);
