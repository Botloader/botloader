import { Discord, HttpClient } from 'botloader';
import { assertExpected, runOnce, sendScriptCompletion } from 'lib';

const TEST_CHANNEL = "531120790318350338"
const TEST_FILE_CONTENT = "gaming wahoo"

runOnce(script.name, async () => {

    const message = await Discord.createMessage(TEST_CHANNEL, {
        content: "Test",
        attachments: [{
            data: TEST_FILE_CONTENT,
            filename: "test.txt",
        }]
    })

    let attachment = message.attachments.find(v => v.filename === "test.txt")
    if (!attachment) {
        throw new Error("attachment not found\n" + JSON.stringify(message.attachments))
    }

    console.log(attachment)
    const resp = await (await HttpClient.get(attachment.url)).text()
    assertExpected(TEST_FILE_CONTENT, resp)

    await message.delete()

    sendScriptCompletion(script.name);
});