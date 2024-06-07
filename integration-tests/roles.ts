import { Discord } from "botloader";
import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const test_role_name_create = "IG_TESTING_ROLE"
const test_role_name_update = "IG_TESTING_ROLE_UPDATE"
const role_perms_create = new Discord.Permissions(
    Discord.Permissions.ChangeNickname,
    Discord.Permissions.Connect
)
const role_perms_update = new Discord.Permissions(
    Discord.Permissions.ChangeNickname,
    Discord.Permissions.Connect,
    Discord.Permissions.KickMembers
)

const deletedRole = script.createStorageVarJson<string>("roles_ts_deleted")

function checkCreatedRole(role: Discord.Role) {
    assertExpected(role.color, 0xff0000)
    assertExpected(role.hoist, true)
    assertExpected(role.permissionsRaw, role_perms_create.toString())
    assertExpected(role.mentionable, false)
}

function checkEditedRole(role: Discord.Role) {
    assertExpected(role.color, 0xff0000)
    assertExpected(role.hoist, false)
    assertExpected(role.permissionsRaw, role_perms_update.toString())
    assertExpected(role.mentionable, false)
}


script.on("ROLE_CREATE", async (role) => {
    if (role.name !== test_role_name_create) {
        return
    }

    checkCreatedRole(role)

    const edited = await Discord.editRole(role.id, {
        name: test_role_name_update,
        hoist: false,
        permissions: role_perms_update
    })

    checkEditedRole(edited)
})


script.on("ROLE_UPDATE", async (role) => {
    if (role.name !== test_role_name_update) {
        return
    }

    checkEditedRole(role)

    deletedRole.set(role.id)

    await Discord.deleteRole(role.id)
})

script.on("ROLE_DELETE", async (evt) => {
    if ((await deletedRole.get())?.value === evt.roleId) {
        sendScriptCompletion(script.name)
    }
})


runOnce(script.name, async () => {
    const created = await Discord.createRole({
        name: test_role_name_create,
        permissions: role_perms_create,
        color: 0xff0000,
        hoist: true,
        mentionable: false,
    })

    checkCreatedRole(created)
})
