import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

runOnce(script.name, async () => {
    const botUser = Discord.getBotUser()

    const messageButons = await Discord.createMessage("531120790318350338", {
        content: `components-buttons`,
        components: [
            new Discord.ActionRow([
                new Discord.CustomButton("super awesome button", "button", "button").setStyle("Primary")
            ]),
        ]
    })

    const messageSelects = await Discord.createMessage("531120790318350338", {
        content: `components`,
        components: [
            new Discord.ActionRow([
                new Discord.SelectMenu("text-select", [
                    new Discord.SelectMenuOption("Option 1", "Value 1"),
                    new Discord.SelectMenuOption("Option 2", "Value 2").setDefault(true),
                    new Discord.SelectMenuOption("Option 3", "Value 3"),
                ])
            ]),
            new Discord.ActionRow([
                new Discord.UserSelectMenu("user-select").setDefaultValues([botUser.id])
            ]),
            new Discord.ActionRow([
                new Discord.RoleSelectMenu("role-select").setDefaultValues(["941078853869260891"])
            ]),
            new Discord.ActionRow([
                new Discord.ChannelSelectMenu("channel-select").setDefaultValues(["531120790318350338"])
            ]),
            new Discord.ActionRow([
                new Discord.MentionableSelectMenu("mentionable-select").setDefaultValues([{ kind: "Role", value: "941078853869260891" }])
            ])
        ]
    })

    assertExpected(1, messageButons.components.length)
    assertExpected(5, messageSelects.components.length)

    sendScriptCompletion(script.name);
})