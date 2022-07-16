import { Commands, Discord } from 'botloader';

script.createCommand(Commands.slashCommand("color-menu", "create a color menu")
    .build(async (ctx) => {
        await ctx.createFollowup({
            content: "Give yourself a color!",