import { HttpClient } from 'botloader';
import { runOnce, sendScriptCompletion } from 'lib';

runOnce(script.name, async () => {
    let resp = await HttpClient.get("http://example.com/");

    sendScriptCompletion(script.name);
})