import { Discord } from 'botloader';
import { assertExpected, runOnce, sendScriptCompletion } from 'lib';

const { Permissions } = Discord;

runOnce("permissions.ts", () => {
    let _ = new Permissions("1");

    let fun = new Permissions(Permissions.AddReactions, Permissions.SendMessages, Permissions.ManageChannels);
    let removed = fun.remove(Permissions.AddReactions, Permissions.SendMessages);
    let addedBack = removed.add(Permissions.AddReactions, Permissions.SendMessages);

    assertExpected((1n << 6n) | 1n << 11n | (1n << 4n), fun.value);
    assertExpected((1n << 4n), removed.value);
    assertExpected((1n << 6n) | 1n << 11n | (1n << 4n), addedBack.value);

    assertExpected(false, fun.hasAny(Permissions.BanMembers, Permissions.ChangeNickname));
    assertExpected(true, fun.hasAny(Permissions.SendMessages, Permissions.ChangeNickname));
    assertExpected(false, removed.hasAny(Permissions.SendMessages, Permissions.ChangeNickname));
    assertExpected(true, addedBack.hasAny(Permissions.SendMessages, Permissions.ChangeNickname));

    assertExpected(false, fun.hasAll(Permissions.BanMembers, Permissions.ChangeNickname));
    assertExpected(false, fun.hasAll(Permissions.SendMessages, Permissions.ChangeNickname));
    assertExpected(false, removed.hasAll(Permissions.SendMessages, Permissions.ChangeNickname));
    assertExpected(false, addedBack.hasAll(Permissions.SendMessages, Permissions.ChangeNickname));

    assertExpected(true, fun.hasAll(Permissions.SendMessages, Permissions.ManageChannels));
    assertExpected(true, removed.hasAll(Permissions.ManageChannels));
    assertExpected(true, addedBack.hasAll(Permissions.SendMessages, Permissions.ManageChannels));

    sendScriptCompletion();
});