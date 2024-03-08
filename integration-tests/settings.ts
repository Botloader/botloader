import { assertExpected, runOnce, sendScriptCompletion } from "lib";

const testStringSettingDefaultRequired = script.settings.addOptionString("string_default_required", {
    defaultValue: "test",
    required: true,
})

const testStringSettingDefaultRequiredProvided = script.settings.addOptionString("string_default_required_provided", {
    defaultValue: "test",
    required: true,
})

const testStringSettingNoDefault = script.settings.addOptionString("string_no_default", {})

const testStringSettingNoDefaultProvided = script.settings.addOptionString("string_no_default_provided", {})


const numberDefaultRequired = script.settings.addOptionFloat("number_default_required", {
    defaultValue: 10,
    required: true,
})

const numberDefaultRequiredProvided = script.settings.addOptionFloat("number_default_required_provided", {
    defaultValue: 10,
    required: true,
})

const numberNoDefault = script.settings.addOptionFloat("number_no_default", {})
const numberNoDefaultProvided = script.settings.addOptionFloat("number_no_default_provided", {})

runOnce("settings", () => {

    assertExpected("test", testStringSettingDefaultRequired.value)
    assertExpected("provided", testStringSettingDefaultRequiredProvided.value)

    assertExpected("", testStringSettingNoDefault.value)
    assertExpected("provided", testStringSettingNoDefaultProvided.value)


    assertExpected(10, numberDefaultRequired.value)
    assertExpected(20, numberDefaultRequiredProvided.value)

    assertExpected(undefined, numberNoDefault.value)
    assertExpected(20, numberNoDefaultProvided.value)

    sendScriptCompletion()
}) 