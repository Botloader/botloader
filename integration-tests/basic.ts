import { runOnce, sendScriptCompletion } from "lib";

runOnce(script.name, () => {
    console.log("fun times");
    sendScriptCompletion(script.name);
})