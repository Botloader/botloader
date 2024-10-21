import { assertExpected, assetJsonEquals, runOnce, sendScriptCompletion } from "lib";
import { Settings } from "botloader";
import { LoadedOption } from "../components/runtime/src/ts/settings";

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

const stringOptions: Settings.StringSelectOptionItem[] = [{
    label: "hello",
    value: "hello value"
}, {
    label: "world",
    value: "world value"
}, {
    label: "third",
    value: "yet another value",
}, {
    label: "fourth",
    value: "fourth value",
}]

const stringSelectDefaultValue = "world value"
const stringSelectProvidedValue = "fourth value"
const stringMultiSelectDefaultValue = ["world value", "fourth value"]
const stringMultiSelectProvidedValue = ["hello value", "yet another value"]

const stringSelectTestCases: SettingsTestCase[] = [
    {
        option: script.settings.addOptionCustomStringSelect("string_select_with_default_not_provided", {
            options: stringOptions,
            defaultValue: stringSelectDefaultValue
        }),
        expected: stringSelectDefaultValue,
    },
    {
        option: script.settings.addOptionCustomStringSelect("string_select_with_default_provided", {
            options: stringOptions,
            defaultValue: stringSelectDefaultValue
        }),
        expected: stringSelectProvidedValue
    },
    {
        option: script.settings.addOptionCustomStringSelect("string_select_no_default_provided", {
            options: stringOptions,
            defaultValue: stringSelectDefaultValue
        }),
        expected: stringSelectProvidedValue
    },
    {

        option: script.settings.addOptionCustomStringSelect("string_select_no_default_not_provided", {
            options: stringOptions,
        }),
        expected: null,
    }
]


const stringMultiSelectTestCases: SettingsTestCase[] = [
    {
        option: script.settings.addOptionCustomStringMultiSelect("str_multiselect_with_default_not_provided", {
            options: stringOptions,
            defaultValue: stringMultiSelectDefaultValue
        }),
        expected: stringMultiSelectDefaultValue,
    },
    {
        option: script.settings.addOptionCustomStringMultiSelect("str_multiselect_with_default_provided", {
            options: stringOptions,
            defaultValue: stringMultiSelectDefaultValue
        }),
        expected: stringMultiSelectProvidedValue
    },
    {
        option: script.settings.addOptionCustomStringMultiSelect("str_multiselect_no_default_provided", {
            options: stringOptions,
            defaultValue: stringMultiSelectDefaultValue
        }),
        expected: stringMultiSelectProvidedValue
    },
    {

        option: script.settings.addOptionCustomStringMultiSelect("str_multiselect_no_default_not_provided", {
            options: stringOptions,
        }),
        expected: [],
    }
]

const numberOptions: Settings.NumberSelectOptionItem[] = [{
    label: "day",
    value: 1,
}, {
    label: "week",
    value: 7,
}, {
    label: "month",
    value: 30,
}, {
    label: "year",
    value: 365,
}]

const numberSelectDefaultValue = 30
const numberSelectProvidedValue = 365
const numberMultiSelectDefaultValue = [30, 7]
const numberMultiSelectProvidedValue = [365, 1]

const numberSelectTestCases: SettingsTestCase[] = [
    {
        option: script.settings.addOptionCustomNumberSelect("number_select_with_default_not_provided", {
            options: numberOptions,
            defaultValue: numberSelectDefaultValue
        }),
        expected: numberSelectDefaultValue,
    },
    {
        option: script.settings.addOptionCustomNumberSelect("number_select_with_default_provided", {
            options: numberOptions,
            defaultValue: numberSelectDefaultValue
        }),
        expected: numberSelectProvidedValue
    },
    {
        option: script.settings.addOptionCustomNumberSelect("number_select_no_default_provided", {
            options: numberOptions,
            defaultValue: numberSelectDefaultValue
        }),
        expected: numberSelectProvidedValue
    },
    {

        option: script.settings.addOptionCustomNumberSelect("number_select_no_default_not_provided", {
            options: numberOptions,
        }),
        expected: null,
    }
]


const numberMultiSelectTestCases: SettingsTestCase[] = [
    {
        option: script.settings.addOptionCustomNumberMultiSelect("num_multiselect_with_default_not_provided", {
            options: numberOptions,
            defaultValue: numberMultiSelectDefaultValue
        }),
        expected: numberMultiSelectDefaultValue,
    },
    {
        option: script.settings.addOptionCustomNumberMultiSelect("num_multiselect_with_default_provided", {
            options: numberOptions,
            defaultValue: numberMultiSelectDefaultValue
        }),
        expected: numberMultiSelectProvidedValue
    },
    {
        option: script.settings.addOptionCustomNumberMultiSelect("num_multiselect_no_default_provided", {
            options: numberOptions,
            defaultValue: numberMultiSelectDefaultValue
        }),
        expected: numberMultiSelectProvidedValue
    },
    {

        option: script.settings.addOptionCustomNumberMultiSelect("num_multiselect_no_default_not_provided", {
            options: numberOptions,
        }),
        expected: [],
    }
]

type SettingsTestCase = {
    option: LoadedOption<any, any, any>,
    expected: any,
}

function testTestCase(testCase: SettingsTestCase) {
    try {
        if (Array.isArray(testCase.expected)) {
            assetJsonEquals(testCase.expected, testCase.option.value)
        } else {
            assertExpected(testCase.expected, testCase.option.value)
        }
    } catch (error: any) {
        throw new Error(`settings test case ${testCase.option.definition.name} failed: ${error.message}`)
    }
}

function testTopLevelOptions() {

    assertExpected("test", testStringSettingDefaultRequired.value)
    assertExpected(expectedProvidedValueString, testStringSettingDefaultRequiredProvided.value)

    assertExpected("", testStringSettingNoDefault.value)
    assertExpected(expectedProvidedValueString, testStringSettingNoDefaultProvided.value)


    assertExpected(10, numberDefaultRequired.value)
    assertExpected(expectedProvidedValueFloat, numberDefaultRequiredProvided.value)

    assertExpected(null, numberNoDefault.value)
    assertExpected(expectedProvidedValueFloat, numberNoDefaultProvided.value)

    stringSelectTestCases.forEach(testTestCase)
    stringMultiSelectTestCases.forEach(testTestCase)
    numberSelectTestCases.forEach(testTestCase)
    numberMultiSelectTestCases.forEach(testTestCase)
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

function testList(expectedValues: any[], input: any[]) {
    for (let i = 0; i < expectedValues.length; i++) {
        const expected = expectedValues[i]
        const value = input[i]
        if (!value) {
            throw new Error(`value at ${i} is false-y`)
        }

        for (let [k, expectedValue] of Object.entries(expected)) {
            let valueEntry = (value as any)[k]
            if (Array.isArray(expectedValue)) {
                expectedValue = JSON.stringify(expectedValue)
                valueEntry = JSON.stringify(valueEntry)
            }

            if (valueEntry !== expectedValue) {
                throw new Error(`incorrect value at value[${i}][${k}], expected '${expectedValue}' got '${valueEntry}'`)
            }
        }
    }
}

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

    testList(expectedValues, testListString.value)
}

const testListStringSelects = script.settings.startList("string_selects_list")
    .addOptionCustomStringSelect("no_default_empty", { options: stringOptions })
    .addOptionCustomStringSelect("no_default_provided", { options: stringOptions })
    .addOptionCustomStringSelect("with_default_empty", { options: stringOptions, defaultValue: stringSelectDefaultValue })
    .addOptionCustomStringSelect("with_default_provided", { options: stringOptions, defaultValue: stringSelectDefaultValue })
    .addOptionCustomStringMultiSelect("multi_no_default_empty", { options: stringOptions })
    .addOptionCustomStringMultiSelect("multi_no_default_provided", { options: stringOptions })
    .addOptionCustomStringMultiSelect("multi_with_default_empty", { options: stringOptions, defaultValue: stringMultiSelectDefaultValue })
    .addOptionCustomStringMultiSelect("multi_with_default_provided", { options: stringOptions, defaultValue: stringMultiSelectDefaultValue })
    .complete()

function testStringSelectLists() {
    const expectedValues = [{
        no_default_empty: null,
        no_default_provided: stringSelectProvidedValue,
        with_default_empty: stringSelectDefaultValue,
        with_default_provided: stringSelectProvidedValue,
        multi_no_default_empty: [],
        multi_no_default_provided: stringMultiSelectProvidedValue,
        multi_with_default_empty: stringMultiSelectDefaultValue,
        multi_with_default_provided: stringMultiSelectProvidedValue,
    }]

    testList(expectedValues, testListStringSelects.value)
}

const testListNumberSelects = script.settings.startList("number_selects_list")
    .addOptionCustomNumberSelect("no_default_empty", { options: numberOptions })
    .addOptionCustomNumberSelect("no_default_provided", { options: numberOptions })
    .addOptionCustomNumberSelect("with_default_empty", { options: numberOptions, defaultValue: numberSelectDefaultValue })
    .addOptionCustomNumberSelect("with_default_provided", { options: numberOptions, defaultValue: numberSelectDefaultValue })
    .addOptionCustomNumberMultiSelect("multi_no_default_empty", { options: numberOptions })
    .addOptionCustomNumberMultiSelect("multi_no_default_provided", { options: numberOptions })
    .addOptionCustomNumberMultiSelect("multi_with_default_empty", { options: numberOptions, defaultValue: numberMultiSelectDefaultValue })
    .addOptionCustomNumberMultiSelect("multi_with_default_provided", { options: numberOptions, defaultValue: numberMultiSelectDefaultValue })
    .complete()

function testNumberSelectLists() {
    const expectedValues = [{
        no_default_empty: null,
        no_default_provided: numberSelectProvidedValue,
        with_default_empty: numberSelectDefaultValue,
        with_default_provided: numberSelectProvidedValue,
        multi_no_default_empty: [],
        multi_no_default_provided: numberMultiSelectProvidedValue,
        multi_with_default_empty: numberMultiSelectDefaultValue,
        multi_with_default_provided: numberMultiSelectProvidedValue,
    }]

    testList(expectedValues, testListNumberSelects.value)
}

// defaultExpected = [}

const testListDefaultSetting = script.settings.startList("list_default")
    .addOptionString("a", { required: true })
    .addOptionBoolean("b", {})
    .complete({
        defaultValue: [{
            a: "a_value_1",
            b: true
        },
        {
            a: "a_value_2",
            b: false
        }]
    })

function testListDefault() {
    const expectedValues = [{
        a: "a_value_1",
        b: true
    },
    {
        a: "a_value_2",
        b: false
    }]

    testList(expectedValues, testListDefaultSetting.value)
}

runOnce(script.name, () => {
    testTopLevelOptions()

    testListOptions()

    testStringSelectLists()
    testNumberSelectLists()

    testListDefault()

    sendScriptCompletion(script.name)

}) 
