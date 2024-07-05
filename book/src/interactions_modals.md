# Modals

Modals shows up a popup window for users to enter text into.

Modals only support text inputs currently, either `ParagraphTextInput` (multi-line) or `ShortTextInput`.

Modals can be triggered from:
- Commands
  - You need to call `setAckMode` with `Custom` and use `ackWithModal`
- Button and all Select menu interactions using `ackWithModal`

Below is an example of a modal triggered from a command:

```ts

import { Commands, Discord } from 'botloader';

script.createSlashCommand("modal", "open a modal")
    .addOptionString("default_text", "default text")
    .setAckMode("Custom")
    .build(
        async (ctx, args) => {
            await ctx.ackWithModal({
                // I haven't added a helper class for modals yet so we have to use the relatively low level
                // encodeInteractionCustomId function to add the custom id
                customId: Discord.encodeInteractionCustomId("test-modal", {}),
                title: "test-modal",
                components: [
                    new Discord.ActionRow([
                        new Discord.ParagraphTextInput("field", "input")
                            .setPlaceHolder(args.default_text ?? "stuff here!")
                            .setValue(args.default_text ?? "stufff"),
                    ])
                ]
            })
        }
    )

script.onInteractionModalSubmit("test-modal", (interaction) => {
    let input = interaction.values["input"];
    interaction.ackWithMessage({ content: "you input: " + input.value });
});

```