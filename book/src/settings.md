# Script / Plugin Settings

Settings allows you to define a set of options that can be edited on the website (and in the future through commands) without having to modify the source code.

In plugins the benefits are pretty obvious, you get to expose configuration options for your plugin that servers can change to their liking.

It can also be useful in server scripts to expose settings to staff members that are not confident in touching the code.

## Settings types

There are a number of settings types:
 - Strings
 - numbers (float, integer, integer64)
 - role and roles (single vs multiple)
 - channel and channels (single vs multiple)

In addition there's the list type that's a bit special, see the list section for more info on that one.

## Defining settings options

Settings options are managed through the `script.settings` field, it's a instance of `SettingsManager`.

To define a option use one of the `addOption*` methods, or `startList` for lists.

Here's some examples:

```ts
const xpNameSetting = script.settings.addOptionString("xp_name", {
    label: "XP point name",
    description: "Name to give xp points",
    defaultValue: "xp",
    required: true,
})

const messageXpCooldownSecondsSetting = script.settings.addOptionInteger("message_xp_cooldown_seconds", {
    label: "Message XP coooldown",
    description: "The time period between messages to wait before they're eligible for more XP",
    defaultValue: 60,
    min: 0,
    required: true,
})

const blacklistedRolesSetting = script.settings.addOptionRoles("blacklisted_roles", {
    label: "Blacklisted Roles",
    description: "Users with these roles do not gain xp",
})

const blacklistedChannelsSetting = script.settings.addOptionChannels("blacklisted_channels", {
    label: "Blacklisted Channels",
    description: "Users do not gain xp in these channels",
})

const levelRolesSetting = script.settings.startList("level_roles")
    .addOptionInteger("level", {
        label: "Level",
        required: true,
        min: 1,
        description: "The level at which the user gains the role"
    })
    .addOptionRole("role", {
        label: "Level",
        required: true,
        description: "The role to assign the user",
        requireAssignable: true,
    }).complete({
        label: "Level Roles",
        description: "Roles to give users as they advance in levels",
    })
```