# Component State

Discord has a mechanism for embedding a limited amount of data within components in a "custom id" field, this is limited to 100 bytes but botloader needs some of for internal matters so you should not expect more than 90 or so bytes available there. This is enough for embedding a couple of id's or something else.

Most component builders accept a last argument for passing in data, below is an example of a counter that keeps the current count in the component state:

```ts
import { Commands, Discord } from 'botloader';

interface CounterState {
    currentCount: number,
}

script.createSlashCommand("create-counter", "create a counter")
    .build(async (ctx) => {
        await ctx.createFollowup(createCountMessage({ currentCount: 0 }));
    })

// we can pass in <CounterState> to tell the function that we expect the data to have the `CounterState` structure
script.onInteractionButton<CounterState>("counter", async (interaction, data) => {
    await interaction.ackWithUpdateMessage(createCountMessage(data))
});


function createCountMessage(state: CounterState): Discord.CreateMessageFields {
    state.currentCount++;
    return {
        content: `count: ${state.currentCount}`,
        components: [
            new Discord.ActionRow([
                // state is embedded into the button here
                new Discord.CustomButton("Increment!", "counter", state).setStyle("Primary"),
            ]),
        ]
    }
}
```

Internally the component builders encodes the data in JSON (might be changed in the future to squeeze more data in) using `encodeInteractionCustomId`.