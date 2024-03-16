import { runOnce, sendScriptCompletion } from "lib";

runOnce("basic.ts", () => {
    console.log("fun times");
    sendScriptCompletion();
})