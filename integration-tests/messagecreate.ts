import { Discord } from 'botloader';
import { runOnce, sendScriptCompletion } from 'lib';

const testMessageContent = "test message integration test"
script.on("MESSAGE_CREATE", (msg) => {
    if (msg.content === testMessageContent) {
        sendScriptCompletion(script.name);
    }
})

runOnce(script.name, async () => {
    Discord.createMessage("531120790318350338", { content: testMessageContent })
})