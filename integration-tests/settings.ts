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

const expectedProvidedValueString = "provided"
const expectedProvidedValueString2 = "provided2"
const expectedProvidedValueFloat = 20

function testTopLevelOptions() {

    assertExpected("test", testStringSettingDefaultRequired.value)
    assertExpected(expectedProvidedValueString, testStringSettingDefaultRequiredProvided.value)

    assertExpected("", testStringSettingNoDefault.value)
    assertExpected(expectedProvidedValueString, testStringSettingNoDefaultProvided.value)


    assertExpected(10, numberDefaultRequired.value)
    assertExpected(expectedProvidedValueFloat, numberDefaultRequiredProvided.value)

    assertExpected(null, numberNoDefault.value)
    assertExpected(expectedProvidedValueFloat, numberNoDefaultProvided.value)
}

const testListString = script.settings.startList("string_list")
    .addOptionString("string_required_no_default_provided", { required: true })
    .addOptionString("string_required_default", { required: true, defaultValue: "default" })
    .addOptionString("string_required_default_provided", { required: true, defaultValue: "default" })
    .addOptionString("string", {})
    .addOptionString("string_provided", {})
    .addOptionString("string_default", { defaultValue: "default" })
    .addOptionString("string_default_provided", { defaultValue: "default" })
    .complete()

function testListOptions() {
    const expectedValues = [{
        string_required_no_default_provided: expectedProvidedValueString,
        string_required_default: "default",
        string_required_default_provided: expectedProvidedValueString,
        string: "",
        string_provided: expectedProvidedValueString,
        string_default: "default",
        string_default_provided: expectedProvidedValueString,
    }, {
        string_required_no_default_provided: expectedProvidedValueString2,
        string_required_default: "default",
        string_required_default_provided: expectedProvidedValueString2,
        string: "",
        string_provided: expectedProvidedValueString2,
        string_default: "default",
        string_default_provided: expectedProvidedValueString2,
    }]

    for (let i = 0; i < expectedValues.length; i++) {
        const expected = expectedValues[i]
        const value = testListString.value[i]
        if (!value) {
            throw new Error(`value at ${i} is false-y`)
        }

        for (const [k, v] of Object.entries(expected)) {
            const valueEntry = (value as any)[k]
            if (valueEntry !== v) {
                throw new Error(`incorrect value at value[${i}][${k}], expected '${v}' got '${valueEntry}'`)
            }
        }
    }
}

runOnce(script.name, () => {
    testTopLevelOptions()

    testListOptions()

    sendScriptCompletion(script.name)
}) 
