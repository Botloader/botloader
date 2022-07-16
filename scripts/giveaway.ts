import { Commands, Discord, Tasks } from 'botloader';

const giveawayIdGenerator = script.createStorageVarNumber("giveaway_id");

const enterEmoji = { unicode: "🎉" };

// We use short identifiers here to save space in the data field since that field is limited to 80 bytes
interface GiveawayInteractionData {
    // Time it ends at
    t: number,
    // Item
    i: string,
    // quantity
    q: number,
};

const group = new Commands.Group("giveaway", "Giveaway commands");
script.createCommand(
    Commands.slashCommand("create", "Create a new giveaway")
        .addOptionString("item", "What's being given away")
        .addOptionString("duration", "how long should the giveaway last? (ex: 3 hours)")
        .addOptionNumber("quantity", "how many winners?", { required: false, minValue: 1, maxValue: 99 })
        .setGroup(group)
        .setAckMode("Custom")
        .build(async (ctx, args) => {
            if (args.item.length > 45) {
                await ctx.ackWithMessage({ content: "Giveaway item can be max 45 characters long", flags: { ephemeral: true } });
                return;
            }

            const quantity = args.quantity ?? 1;

            const when = (parseTime(args.duration) * 1000);
            if (when < 60000) {
                await ctx.ackWithMessage({ content: "The giveaway has to last atleast 1 minute, example: `3 hours 10 minuttes`", flags: { ephemeral: true } });
                return;
            }

            const endsAt = new Date(Date.now() + (parseTime(args.duration) * 1000));

            const unixTimeSeconds = Math.floor(endsAt.getTime() / 1000);
            const data: GiveawayInteractionData = { t: unixTimeSeconds, i: args.item, q: quantity };
            await ctx.ackWithMessage({
                content: `Do you want to start a giveaway for ${formatItem(args.item, quantity)} in <t:${unixTimeSeconds}:R> at <t:${unixTimeSeconds}>? Dismiss this message to abort.`,
                flags: { ephemeral: true },
                components: [
                    new Discord.ActionRow([
                        new Discord.CustomButton("Create", "gy_new", data),
                    ])
                ],
            })
        })
)

function parseTime(s: string) {
    console.log(s);
    let secondsUntil = 0;
    let split = s.split(" ");

    for (let i = 0; i < split.length; i++) {
        if (split[i].length === 0 || split[i].toLowerCase() === "in") {
            continue
        }


        let [quantity, unit] = parseQuantityUnit(split[i]);

        if (unit === "") {
            i++;
            unit = split[i];
        }

        // read the unit
        switch (unit.toLowerCase()) {
            case "m":
            case "min":
            case "mins":
            case "minute":
            case "minutes":
                secondsUntil += quantity * 60
                break;
            case "h":
            case "hour":
            case "hours":
                secondsUntil += quantity * 60 * 60
                break;
            case "d":
            case "day":
            case "days":
                secondsUntil += quantity * 60 * 60 * 24
                break;
            case "w":
            case "week":
            case "weeks":
                secondsUntil += quantity * 60 * 60 * 24 * 7
                break;
            default:
                // we just ignore unknow units
                break;
        }
    }

    return secondsUntil;
}

function parseQuantityUnit(s: string): [number, string] {
    let numStr = "";
    let rest = "";

    // for .. of uses the string's iterator
    // so we correctly iterate over full characters
    // and NOT 16 bit codepoints (altough that's not really important here as we only suppor english but whatever)
    for (let char of s) {
        if (rest !== "") {
            rest += char;
            continue;
        }

        if (char === "0" || char === "1" || char === "2" || char === "3" || char === "4"
            || char === "5" || char === "6" || char === "7" || char === "8" || char === "9") {
            numStr += char;
        } else {
            rest += char;
        }
    }

    let parsed = parseInt(numStr);
    return [isNaN(parsed) ? 1 : parsed, rest];
}

script.onInteractionButton<GiveawayInteractionData>("gy_new",
    async (ctx, data) => {
        const giveawayId = (await giveawayIdGenerator.incr(1)).value;

        await ctx.ackWithUpdateMessage({ content: `created giveaway #${giveawayId}!`, components: [] });

        let resp = await ctx.createFollowup(giveawayMessage(giveawayId, data.i, data.q, data.t, "Active"));

        const endAt = new Date(data.t * 1000);
        const task = await Tasks.schedule("giveaways", endAt, {
            key: giveawayId + "",
            data: {
                item: data.i,
                authorId: ctx.member.user.id,
                quantity: data.q,
                channelId: resp.channelId,
                messageId: resp.id,
            },
        });
        await Discord.createReaction(resp.channelId, resp.id, enterEmoji);

        console.log("Set up giveaway #" + giveawayId, task);
    }
)

function giveawayMessage(id: number, item: string, quantity: number, endsAtUnixSeconds: number, state: "Active" | "Ending" | "Ended", authorId?: string, winners?: string[],): Discord.CreateMessageFields {

    let details = `React with ${enterEmoji.unicode} below to join!`;
    if (state == "Ending") {
        details = "Giveaway is ending, calculating winners..."
    } else if (state === "Ended") {
        details = "Giveaway has ended"
    }

    let winnersMsg = "";
    if (state === "Ended") {
        if (winners && winners.length > 0) {
            winnersMsg = "Congrats to the winner(s):";
            for (let winner of winners) {
                winnersMsg += ` <@${winner}>`
            }
        } else {
            winnersMsg = "Congrats to no one for winning! No one entered the giveaway! woooo!"
        }
    }

    let components: Discord.IComponent[] = [];
    if (state === "Ended") {
        components = [
            new Discord.ActionRow([
                new Discord.CustomButton("Re-roll", "gy_reroll", { q: quantity, u: authorId })
            ])
        ]
    }

    return {
        content: winnersMsg,
        components: components,
        embeds: [{
            title: `Giveaway #${id}`,
            description: `Giveaway for **${formatItem(item, quantity)}**!
${details}`,
            fields: [{
                name: "Ends at",
                value: `<t:${endsAtUnixSeconds}> - <t:${endsAtUnixSeconds}:R>`
            }],
        }]
    }
}

function formatItem(item: string, quantity: number) {
    const itemMsgQuantifier = quantity > 1 ? `${quantity}x ` : "";
    return itemMsgQuantifier + item;
}

interface TaskData {
    item: string,
    authorId: string
    quantity: number,
    channelId: string,
    messageId: string,
}

script.onTask<TaskData>("giveaways", async (task) => {
    console.log("Ending giveaway: ", task.data)
    await endGiveaway(parseInt(task.key!), task.data, Math.floor(task.executeAt / 1000));
})

async function endGiveaway(id: number, task: TaskData, endsAtUnixTimeSeconds: number) {
    // mark it as ending
    await Discord.editMessage(task.channelId, task.messageId,
        giveawayMessage(id, task.item, task.quantity, endsAtUnixTimeSeconds, "Ending"));

    // download a list of all the participants
    const winners = await pickWinners(task.channelId, task.messageId, task.quantity);

    // update the giveaway message with results
    await Discord.editMessage(task.channelId,
        task.messageId,
        giveawayMessage(id, task.item, task.quantity, endsAtUnixTimeSeconds, "Ended", task.authorId, winners)
    );
}

async function pickWinners(channelId: string, messageId: string, quantity: number) {
    // download a list of all the participants
    let allReactions = await downloadReactions(channelId, messageId);

    // pick winners
    let winnerIndexes: number[] = [];
    if (allReactions.length <= quantity) {
        // everyone who entered won
        for (let i = 0; i < allReactions.length; i++) {
            winnerIndexes.push(i);
        }
    } else {
        for (let i = 0; i < quantity; i++) {
            let candidate = Math.floor(Math.random() * allReactions.length);
            if (winnerIndexes.find(v => v === candidate)) {
                // winner already selected
                i--;
                continue;
            } else {
                winnerIndexes.push(candidate);
            }
        }
    }

    return winnerIndexes.map(v => allReactions[v].toString())

}

async function downloadReactions(channelId: string, messageId: string) {
    let after: string | undefined = undefined;

    // we use a Bigint64Array for optimisation purposes
    // using this we can go well beyond 100k users without issues
    let results = new BigInt64Array();
    while (true) {
        let resp = await Discord.getReactions(channelId, messageId, enterEmoji, { after: after, limit: 100 });
        if (resp.length > 0) {
            // why do we do a string? well typescript completely breaks if you don't, try it...
            after = resp[resp.length - 1].id as string;
        }

        // filter out bots
        const filtered = resp.filter(v => !v.bot);
        if (filtered.length > 0) {
            const newResults = new BigInt64Array(results.length + filtered.length)
            newResults.set(results, 0);

            for (let i = 0; i < filtered.length; i++) {
                newResults[results.length + i] = BigInt(filtered[i].id);
            }

            results = newResults;
        }

        if (resp.length < 100) {
            break;
        }
    }

    return results
}

interface RerollButtonData {
    // author id
    u: string,
    // quantity
    q: number,
}


script.onInteractionButton<RerollButtonData>("gy_reroll",
    async (ctx, data) => {
        if (ctx.member.user.id !== data.u) {
            await ctx.ackWithMessage({
                content: "only the author of the giveaway can re-roll",
                flags: { ephemeral: true }
            })
            return
        }

        await ctx.ackWithUpdateMessage({ content: ctx.message.content + "\nre-rolling winners..." });

        // download a list of all the participants
        const winners = await pickWinners(ctx.channelId, ctx.message.id, data.q);

        let winnersMsg = "Rerolled: Congrats to the new winner(s):";
        for (let winner of winners) {
            winnersMsg += ` <@${winner}>`
        }

        await ctx.editOriginalResponse({
            embeds: ctx.message.embeds,
            components: ctx.message.components,
            content: ctx.message.content + "\n" + winnersMsg
        })
    }
)