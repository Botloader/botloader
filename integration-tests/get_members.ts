import { Discord, HttpClient } from 'botloader';
import { runOnce, sendScriptCompletion } from 'lib';

runOnce("get_members.ts", async () => {
    let result = await Discord.getMembers(["852626272794181692", "105487308693757952", "204255221017214977"])
    for (const item of result) {
        if (!item) {
            throw new Error(`One or more members failed to fetch`);
        }
    }

    sendScriptCompletion();
})