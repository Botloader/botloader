# Handling interactions

Use one of the `script.onInteraction*` functions to register a handler for a component with the provided name.

To send a response you can either:
- Update the message that triggered the interaction using `ackWithUpdateMessage` or `ackWithDeferredUpdateMessage`
- Send a new message using `ackWithMessage` or `ackWithDeferredMessage`

In the interaction handler you need to use one of the `ack*` functions to "acknowledge" the interaction within 3 seconds, if it might take longer you can use either `interaction.ackWithDeferredUpdateMessage` or `interaction.ackWithDeferredMessage` to tell discord you acknowledge it but need more time to send proper response.

Components can also  have additional data embedded in them, you can read more about that in the [component state](./interactions_state.md) section.