import { Discord } from 'botloader';
import { runOnce, sendScriptCompletion } from 'lib';

runOnce("permissions.ts", () => {
    let perms = new Discord.Permissions("1");

    // let new_perms = perms.add(1);
    // if (true){
    //     // can't do this
    //     // new_perms = new_perms.add(1)


    //     new_perms = new Discord.Permissions(new_perms).add(1)
    // }

    sendScriptCompletion();
});