import { Commands } from "botloader";

const tags = script.createStorageJson<{
    content: string,
    ownerId: string,
    uses: number,
    createdAt: number,
}>("tags");

const RESERVED_NAMES = ["help", "list", "create", "delete", "info", "edit"];

function validateTagName(name: string): string | null {
    if (name.length < 2 || name.length > 32) {
        return "tag names must be between 2 and 32 characters";
    }
    if (!/^[a-z0-9-_]+$/.test(name)) {
        return "tag names can only contain lowercase letters, numbers, dashes and underscores";
    }
    if (RESERVED_NAMES.includes(name)) {
        return "that name is reserved";
    }
    return null;
}

script.createCommand(
    Commands.slashCommand("tag", "show a tag")
        .addOptionString("name", "name of the tag")
        .build(async (ctx, args) => {
            const tag = await tags.get(args.name);
            if (!tag) {
                await ctx.sendResponse(`no tag named '${args.name}'`);
                return;
            }
            await tags.set(args.name, { ...tag.value, uses: tag.value.uses + 1 });
            await ctx.sendResponse(tag.value.content);
        })
);

script.createCommand(
    Commands.slashCommand("tag-create", "create a new tag")
        .addOptionString("name", "name of the tag")
        .addOptionString("content", "what the tag should say")
        .build(async (ctx, args) => {
            const err = validateTagName(args.name);
            if (err) {
                await ctx.sendResponse(err);
                return;
            }

            const created = await tags.setIf(args.name, {
                content: args.content,
                ownerId: ctx.member.user.id,
                uses: 0,
                createdAt: Date.now(),
            }, "IfNotExists");

            await ctx.sendResponse(created ? `tag '${args.name}' created` : "that tag already exists");
        })
);

script.createCommand(
    Commands.slashCommand("tag-delete", "delete one of your tags")
        .addOptionString("name", "name of the tag")
        .build(async (ctx, args) => {
            const tag = await tags.get(args.name);
            if (!tag) {
                await ctx.sendResponse("no such tag");
                return;
            }
            if (tag.value.ownerId !== ctx.member.user.id) {
                await ctx.sendResponse("you don't own that tag");
                return;
            }
            await tags.del(args.name);
            await ctx.sendResponse("tag deleted");
        })
);

script.createCommand(
    Commands.slashCommand("tag-info", "show info about a tag")
        .addOptionString("name", "name of the tag")
        .build(async (ctx, args) => {
            const tag = await tags.get(args.name);
            if (!tag) {
                await ctx.sendResponse("no such tag");
                return;
            }
            await ctx.sendResponse(
                `'${args.name}' owned by <@${tag.value.ownerId}>, used ${tag.value.uses} times`
            );
        })
);
