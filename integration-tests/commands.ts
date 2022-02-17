import { Commands } from 'botloader';
import { runOnce, sendScriptCompletion } from 'lib';

script.addCommand(
    Commands.slashCommand("gaming", "this is a gaming command")
        .addOptionNumber("amount", "amount of gaming")
        .build((ctx, args) => {
            // stuff here
            let a = args.amount;
            ctx.sendResponse(`we are gaming: ${a}`);
        })
);


script.addCommand(
    Commands.userCommand("throw", "throw this user up in the air")
        .build((ctx, target) => {
            // stuff here
            ctx.sendResponse(`throwing ${target.user.id}`);
        })
);


script.addCommand(
    Commands.messageCommand("report", "report this message")
        .build((ctx, target) => {
            // stuff here
            ctx.sendResponse(`reporing ${target.id} made by ${target.author.id}`);
        })
);

runOnce("commands.ts", () => {
    sendScriptCompletion();
});