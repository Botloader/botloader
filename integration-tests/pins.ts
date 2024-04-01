import { Discord, Tasks } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const channelId = "959455076898975884";
const messageId1 = "959455086885609562";
const messageId2 = "959455089154728006";
runOnce(script.name, async () => {

    await Discord.createPin(channelId, messageId1);
    await Discord.createPin(channelId, messageId2);

    let pins = await Discord.getPins(channelId);
    assertExpected(2, pins.length);

    await Discord.deletePin(channelId, messageId1);
    await Discord.deletePin(channelId, messageId2);

    Tasks.schedule("pins.ts_1", new Date(Date.now() + 10_000));
})

script.onTask("pins.ts_1", async (t) => {
    // needs to be a task for glithcy ratleimiting reasons
    let pins = await Discord.getPins(channelId);
    assertExpected(0, pins.length);

    sendScriptCompletion(script.name)
})