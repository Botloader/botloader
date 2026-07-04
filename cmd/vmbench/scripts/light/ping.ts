import { Commands } from "botloader";

script.createCommand(
    Commands.slashCommand("ping", "check if the bot is alive")
        .build(async (ctx) => {
            await ctx.sendResponse("pong!");
        })
);

script.on("MESSAGE_CREATE", (msg) => {
    if (msg.content === "ping") {
        console.log("someone said ping");
    }
});
