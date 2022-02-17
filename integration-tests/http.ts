import { HttpClient } from 'botloader';
import { runOnce, sendScriptCompletion } from 'lib';

runOnce("http.ts", async () => {
    let resp = await HttpClient.get("http://example.com/");

    sendScriptCompletion();
})