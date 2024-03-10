# Settings List Option

The list option type, accessed through `script.settings.startList` is a special type that allows users to provide you with a list of values.

To use it you define the options that each list item has, sort of like a "schema" for the list items.

You add options to the list using the `addOption` methods on the list builder and complete it using the `complete` method.

Unlike with top level options you can mark options in the list as required without providing a default value, users will be prevented to save the new list items until they provide all the required values.

Here's an example of using the list option type for a level roles feature of a xp system:
```ts
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

console.log(`Defined level roles: ${JSON.stringify(levelRolesSetting.value)}`)
```