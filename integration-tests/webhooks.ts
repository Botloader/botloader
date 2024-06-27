import { Discord } from "botloader";
import { assertExpected, assetJsonEquals, runOnce, sendScriptCompletion } from "lib";

const testChannelId = "1255906191347814441"

runOnce(script.name, async () => {
    // Test creation
    const hook = await Discord.createWebhook({
        channel_id: testChannelId,
        name: "ig_test_webhook",
    })

    assertExpected("ig_test_webhook", hook.name)

    // Test editing
    const hook2 = await hook.edit({
        name: "ig_test_webhook_edit"
    })

    assertExpected("ig_test_webhook_edit", hook2.name)

    // Test listing
    const channelWebhooks = await Discord.getChannelWebhooks(testChannelId)
    const found = channelWebhooks.find(v => v.id === hook.id)
    assetJsonEquals(hook2, found)

    // Test executing
    const message = await hook2.execute({
        content: "awesome"
    })

    assertExpected("awesome", message.content)
    assertExpected(hook2.id, message.webhookId)

    // Test editing webhook message
    const editedMessage = await Discord.editWebhookMessage(
        hook2.id,
        hook2.token!,
        message.id,
        {
            content: "awesome edited"
        }
    )

    assertExpected("awesome edited", editedMessage.content)

    // Test deleting webhook message
    await Discord.deleteWebhookMessage(hook2.id, hook2.token!, message.id)

    // Test deleting webhook
    await hook2.delete()

    // Ensure its deleted
    const channelWebhooksPostDelete = await Discord.getChannelWebhooks(testChannelId)
    const foundPostDelete = channelWebhooksPostDelete.find(v => v.id === hook.id)
    assertExpected(undefined, foundPostDelete)

    sendScriptCompletion(script.name)
})