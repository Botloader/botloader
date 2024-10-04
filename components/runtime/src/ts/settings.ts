import { OpWrappers } from "./op_wrappers"
import * as Internal from "./generated/internal/index";
import { ChannelType } from "./discord/index";

export type DefaultValue<TDefault> = TDefault extends undefined
    ? { defaultValue?: TDefault }
    : { defaultValue: TDefault }

export type BaseOptions<TRequired, TDefault> = {
    /**
     * A cosmetic label for this option, used when displaying it
     * 
     * Changing the label will not alter the user provided value in any way unlike changing the "name" for the option. 
     */
    label?: string
    description?: string

    /**
     * Whether this setting is required, this can only be set on top level options if a default value is provided.
     */
    required?: TRequired
} & DefaultValue<TDefault>

export type StringOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    /**
     * Max length of the string in unicode runes
     */
    maxLength?: number,

    /**
     * Minimum length of the string in unicode runes
     * 
     * Note if you just want to disallow an empty value then you should set "required" instead
     */
    minLength?: number,
}

export type FloatOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    /**
     * Maximum allowed value
     */
    max?: number,

    /**
     * Minimum allowed value
     */
    min?: number,
}

export type IntegerOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    /**
     * Maximum allowed value
     */
    max?: number,

    /**
     * Minimum allowed value
     */
    min?: number,
}

export type Integer64Options<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    /**
     * Maximum allowed value
     */
    max?: string,

    /**
     * Minimum allowed value
     */
    min?: string,
}

export type ChannelOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    /**
     * What channel types to allow
     */
    types?: ChannelType[]
}

export type ChannelsOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    /**
     * What channel types to allow
     */
    types?: ChannelType[]

    /**
     * Maximum number of channels
     */
    max?: number,
}

export type RoleOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    requireAssignable?: boolean
}

export type RolesOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    requireAssignable?: boolean

    /**
     * Maximum number of roles
     */
    max?: number,
}

export type CustomStringSelectOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    options: StringSelectOptionItem[],
}

export type CustomStringMultiSelectOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    options: StringSelectOptionItem[],
    max_selected?: number,
    min_selected?: number,
}

export type StringSelectOptionItem = {
    label: string,
    value: string,
}

export type CustomNumberSelectOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    options: NumberSelectOptionItem[],
}

export type CustomNumberMultiSelectOptions<TRequired, TDefault> = BaseOptions<TRequired, TDefault> & {
    options: NumberSelectOptionItem[],
    max_selected?: number,
    min_selected?: number,
}

export type NumberSelectOptionItem = {
    label: string,
    value: number,
}

interface OptionTypes<TRequired, TDefault> {
    string: StringOptions<TRequired, TDefault>,
    float: FloatOptions<TRequired, TDefault>,
    integer: IntegerOptions<TRequired, TDefault>,
    integer64: Integer64Options<TRequired, TDefault>,
    boolean: BaseOptions<TRequired, TDefault>,
    channel: ChannelOptions<TRequired, TDefault>,
    channels: ChannelsOptions<TRequired, TDefault>,
    role: RoleOptions<TRequired, TDefault>,
    roles: RolesOptions<TRequired, TDefault>,
    customStringSelect: CustomStringSelectOptions<TRequired, TDefault>,
    customStringMultiSelect: CustomStringMultiSelectOptions<TRequired, TDefault>,
    customNumberSelect: CustomNumberSelectOptions<TRequired, TDefault>,
    customNumberMultiSelect: CustomNumberMultiSelectOptions<TRequired, TDefault>,
}

type OptionTypesKeys = keyof OptionTypes<boolean, any>

type OptionTypesUnion = {
    [Key in OptionTypesKeys]: { kind: Key } & OptionTypes<boolean, any>[Key]
}[OptionTypesKeys]

export interface ConfigOptionSetValueTypesMapping {
    string: string,
    integer: number,
    integer64: string,
    float: number,
    boolean: boolean,
    channel: string,
    channels: string[],
    role: string,
    roles: string[],
    customStringSelect: string,
    customStringMultiSelect: string[],
    customNumberSelect: number,
    customNumberMultiSelect: number[],
}

type InnerToConfigSetValueType<TKind extends OptionTypesKeys> =
    ConfigOptionSetValueTypesMapping[TKind]

// Converts the input generic parameters (kind, default value, required)
// to the value type depending on if the config option is required or has a default value
type ToConfigValue<
    TKind extends OptionTypesKeys,
    TDefaultValue,
    TRequired extends boolean
> =
    TRequired extends true
    ? InnerToConfigSetValueType<TKind>
    : TDefaultValue extends undefined
    ? InnerToConfigSetValueType<TKind> | SysDefaultValueType<TKind>
    : InnerToConfigSetValueType<TKind> | TDefaultValue | SysDefaultValueType<TKind>

/**
 * Builtin default values for option types
 */
export const SysDefaultValues = {
    string: "",
    float: null,
    integer: null,
    integer64: null,
    boolean: false,
    channel: null,
    channels: [] as string[],
    role: null,
    roles: [] as string[],
    customStringSelect: null,
    customStringMultiSelect: [] as string[],
    customNumberSelect: null,
    customNumberMultiSelect: [] as number[],
}

type SysDefaultValueType<TKind extends keyof typeof SysDefaultValues> =
    (typeof SysDefaultValues)[TKind]

function getSysDefaultValue<TKind extends keyof typeof SysDefaultValues>
    (kind: TKind): SysDefaultValueType<TKind> {
    return SysDefaultValues[kind]
}


type OptionDefinitionOption = {
    kind: "Option",
    name: string,
    definition: OptionTypesUnion,
}

type OptionDefinitionList = {
    kind: "List",
    name: string,
    template: Record<string, OptionTypesUnion>,
    options: ListOptions<any>,
}

type AnyOptionDefinition = OptionDefinitionOption | OptionDefinitionList

type ListOptions<TDefault> = {
    description?: string,
    label?: string,
    max?: number,
    min?: number,
    defaultValue?: TDefault,
}

/**
 * A loaded settings option, to access the value use the {@link value} field
 */
export class LoadedOption<TKind extends OptionTypesKeys, TDefaultValue, TRequired extends boolean> {
    definition: OptionDefinitionOption

    private _value: ToConfigValue<TKind, TDefaultValue, TRequired>

    /**
     * The loaded settings value
     */
    get value(): ToConfigValue<TKind, TDefaultValue, TRequired> {
        return this._value;
    }

    /**
     * @internal
     */
    constructor(definition: OptionDefinitionOption, rawValue?: Internal.SettingsOptionValue) {
        this.definition = definition

        if (rawValue) {
            this._value = loadOptionValue(rawValue.value, definition.definition)
        } else {
            this._value = definition.definition.defaultValue ?? getSysDefaultValue(definition.definition.kind)
        }
    }
}

type TDefaultValueTopLevel<TRequired extends boolean, TKind extends OptionTypesKeys> =
    TRequired extends true
    ? InnerToConfigSetValueType<TKind>
    : InnerToConfigSetValueType<TKind> | undefined;

/**
 * Configuration options for your script/plugins
 * 
 * Configuration options are configurable through the web interface, giving users of your script/plugin
 * the ability to change variables though the configuration options youe expose, without having to edit the source code.
 * 
 * To register an optoin you use the appropiate addOption* methods, or the startList method.
 * 
 * **Default values and 'required'**
 * 
 * You can provide a default value that is used when the field is not provided but
 * users can still put it to to the types appropiate empty value unless you set "required" to true
 * 
 * Top level options can only be required if you have a default value assigned to them.
 * 
 * The empty value for strings is an empty string ("") and setting required to true will require a non-empty string.
 * 
 * Example:
 * ```
 * const xpNameSetting = script.settings.addOptionString("xp_name", {
 *     label: "XP point name",
 *     description: "Name to give xp points",
 *     defaultValue: "xp",
 *     required: true,
 * })
 *
 * const messageXpCooldownSecondsSetting = script.settings.addOptionInteger("message_xp_cooldown_seconds", {
 *     label: "Message XP coooldown",
 *     description: "The time period between messages to wait before they're eligible for more XP",
 *     defaultValue: 60,
 *     min: 0,
 *     required: true,
 * })
 * ```
 */
export class SettingsManager {
    /**
     * @internal
     */
    definitions: AnyOptionDefinition[] = [];

    /**
     * @internal
     */
    loadedSettings: Internal.SettingsOptionValue[];

    /**
     * @internal
     */
    constructor(scriptId: number) {
        this.loadedSettings = OpWrappers.getSettings(scriptId)
    }

    addOptionString<
        TDefault extends TDefaultValueTopLevel<TRequired, "string">,
        TRequired extends boolean = false,
    >(name: string, options: StringOptions<TRequired, TDefault>) {
        return this.addOption<"string", TRequired, TDefault>(name, "string", options)
    }

    addOptionFloat<
        TDefault extends TDefaultValueTopLevel<TRequired, "float">,
        TRequired extends boolean = false,
    >(name: string, options: FloatOptions<TRequired, TDefault>) {
        return this.addOption<"float", TRequired, TDefault>(name, "float", options)
    }

    addOptionInteger<
        TDefault extends TDefaultValueTopLevel<TRequired, "integer">,
        TRequired extends boolean = false,
    >(name: string, options: IntegerOptions<TRequired, TDefault>) {
        return this.addOption<"integer", TRequired, TDefault>(name, "integer", options)
    }

    addOptionInteger64<
        TDefault extends TDefaultValueTopLevel<TRequired, "string">,
        TRequired extends boolean = false,
    >(name: string, options: Integer64Options<false, TDefault>) {
        return this.addOption<"integer64", false, TDefault>(name, "integer64", options)
    }

    addOptionBoolean<
        TDefault extends TDefaultValueTopLevel<TRequired, "boolean">,
        TRequired extends boolean = false,
    >(name: string, options: BaseOptions<false, TDefault>) {
        return this.addOption<"boolean", false, TDefault>(name, "boolean", options)
    }

    addOptionRole<
        TDefault extends string | undefined = undefined,
    >(name: string, options: RoleOptions<false, TDefault>) {
        return this.addOption<"role", false, TDefault>(name, "role", options)
    }

    addOptionRoles<
        TDefault extends string[] | undefined = undefined,
    >(name: string, options: RolesOptions<false, TDefault>) {
        return this.addOption<"roles", false, TDefault>(name, "roles", options)
    }

    addOptionChannel<
        TDefault extends string | undefined = undefined,
    >(name: string, options: ChannelOptions<false, TDefault>) {
        return this.addOption<"channel", false, TDefault>(name, "channel", options)
    }

    addOptionChannels<
        TDefault extends string[] | undefined = undefined,
    >(name: string, options: ChannelsOptions<false, TDefault>) {
        return this.addOption<"channels", false, TDefault>(name, "channels", options)
    }

    addOptionCustomStringSelect<
        TDefault extends string | undefined = undefined,
    >(name: string, options: CustomStringSelectOptions<false, TDefault>) {
        return this.addOption<"customStringSelect", false, TDefault>(name, "customStringSelect", options)
    }

    addOptionCustomStringMultiSelect<
        TDefault extends string[] | undefined = undefined,
    >(name: string, options: CustomStringMultiSelectOptions<false, TDefault>) {
        return this.addOption<"customStringMultiSelect", false, TDefault>(name, "customStringMultiSelect", options)
    }

    addOptionCustomNumberSelect<
        TDefault extends number | undefined = undefined,
    >(name: string, options: CustomNumberSelectOptions<false, TDefault>) {
        return this.addOption<"customNumberSelect", false, TDefault>(name, "customNumberSelect", options)
    }

    addOptionCustomNumberMultiSelect<
        TDefault extends number[] | undefined = undefined,
    >(name: string, options: CustomNumberMultiSelectOptions<false, TDefault>) {
        return this.addOption<"customNumberMultiSelect", false, TDefault>(name, "customNumberMultiSelect", options)
    }

    private addOption<
        const TKind extends OptionTypesKeys,
        const TRequired extends boolean = false,
        const TDefault extends InnerToConfigSetValueType<TKind> | undefined = undefined,
    >(name: string, kind: TKind, options: OptionTypes<TRequired, TDefault>[TKind]): LoadedOption<TKind, TDefault, TRequired> {
        const definition: OptionDefinitionOption = {
            kind: "Option",
            name,
            definition: { kind, ...options } as any,
        }

        this.definitions.push(definition)

        const value = this.loadedSettings.find(v => v.name === name)
        return new LoadedOption(definition, value)
    }

    /**
     * Start building a list.
     * 
     * The list give you the ability to expose a option that users can add multiple items to
     * 
     * To create a list you define the schema, a set of options each item in the list needs to have.
     * 
     * An example for this is level roles for a leveling system
     * where each item would have a level integer option and a role option:
     * 
     * ```
     * const levelRolesSetting = script.settings.startList("level_roles")
     *   .addOptionInteger("level", {
     *       label: "Level",
     *       required: true,
     *       min: 1,
     *       description: "The level at which the user gains the role"
     *   })
     *   .addOptionRole("role", {
     *       label: "Level",
     *       required: true,
     *       description: "The role to assign the user"
     *   }).complete({
     *       label: "Level Roles",
     *       description: "Roles to give users as they advance in levels",
     *   })
     * ```
     */
    startList(name: string) {
        return new ListBuilder<{}>(name, this)
    }

    /**
     * @internal
     */
    toInternalOptions(): Internal.SettingsOptionDefinition[] {
        return this.definitions.map(def => defToInternal(def))
    }
}

interface OptionsMap {
    [key: string]: { defaultValue: any, kind: any, required: boolean }
}

type LayerOption<TInner, TKey extends string, TKind extends keyof ConfigOptionSetValueTypesMapping, TDefault, TRequired> =
    { [Prop in keyof TInner]: TInner[Prop] } & { [Prop in TKey]: { defaultValue: TDefault, kind: TKind, required: TRequired } };

type OptionMapValues<T extends OptionsMap> = {
    [Prop in keyof T]: ToConfigValue<
        T[Prop]["kind"],
        T[Prop]["defaultValue"],
        T[Prop]["required"]
    >
}

/**
 * A loaded settings option list, to access the value use the {@link value} field
 */
export class LoadedList<TOpts extends OptionsMap> {
    definition: OptionDefinitionList

    private _value: OptionMapValues<TOpts>[]

    /**
     * The actual loaded settings value
     */
    get value(): OptionMapValues<TOpts>[] {
        return this._value;
    }

    /**
     * @internal
     */
    constructor(definition: OptionDefinitionList, rawValue?: Internal.SettingsOptionValue) {
        this.definition = definition

        if (!rawValue?.value) {
            this._value = definition.options.defaultValue ?? []
            return
        }

        const arrValue = rawValue?.value
        if (!Array.isArray(arrValue)) {
            this._value = definition.options.defaultValue ?? []
            return
        }

        this._value = []
        OUTER:
        for (const entry of arrValue) {
            if (!Array.isArray(entry)) {
                console.log(`Invalid list item in option list ${definition.name} skipping...`)
                continue
            }

            const obj = {} as any
            for (const [name, optionDefinition] of Object.entries(definition.template)) {
                const optionValue = entry.find(v => "name" in v && v.name === name)
                if (optionValue && "value" in optionValue) {
                    obj[name] = loadOptionValue(optionValue.value, optionDefinition)
                } else {
                    if (optionDefinition.required && optionDefinition.defaultValue === undefined) {
                        console.log(`List item in option list ${definition.name} missing required option [${name}], skipping...`)
                        continue OUTER
                    } else {
                        obj[name] = optionDefinition.defaultValue ?? getSysDefaultValue(optionDefinition.kind)
                    }
                }
            }

            this._value.push(obj)
        }
    }
}

export class ListBuilder<TOpts extends OptionsMap> {
    manager: SettingsManager;
    name: string;
    definitions: Record<string, OptionTypesUnion> = {};

    constructor(name: string, manager: SettingsManager) {
        this.name = name
        this.manager = manager
    }

    addOptionString<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: StringOptions<TRequired, TDefault>) {
        return this.addOption<TName, "string", TRequired, TDefault>(name, "string", options)
    }

    addOptionFloat<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends number | undefined = undefined,
    >(name: TName, options: FloatOptions<TRequired, TDefault>) {
        return this.addOption<TName, "float", TRequired, TDefault>(name, "float", options)
    }

    addOptionInteger<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends number | undefined = undefined,
    >(name: TName, options: IntegerOptions<TRequired, TDefault>) {
        return this.addOption<TName, "integer", TRequired, TDefault>(name, "integer", options)
    }

    addOptionInteger64<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: Integer64Options<TRequired, TDefault>) {
        return this.addOption<TName, "integer64", TRequired, TDefault>(name, "integer64", options)
    }

    addOptionBoolean<
        TName extends string,
        TDefault extends boolean | undefined = undefined,
    >(name: TName, options: BaseOptions<false, TDefault>) {
        return this.addOption<TName, "boolean", false, TDefault>(name, "boolean", options)
    }

    addOptionRole<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: RoleOptions<TRequired, TDefault>) {
        return this.addOption<TName, "role", TRequired, TDefault>(name, "role", options)
    }

    addOptionRoles<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string[] | undefined = undefined,
    >(name: TName, options: RolesOptions<TRequired, TDefault>) {
        return this.addOption<TName, "roles", TRequired, TDefault>(name, "roles", options)
    }

    addOptionChannel<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: ChannelOptions<TRequired, TDefault>) {
        return this.addOption<TName, "channel", TRequired, TDefault>(name, "channel", options)
    }

    addOptionChannels<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string[] | undefined = undefined,
    >(name: TName, options: ChannelsOptions<TRequired, TDefault>) {
        return this.addOption<TName, "channels", TRequired, TDefault>(name, "channels", options)
    }

    addOptionCustomStringSelect<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string | undefined = undefined,
    >(name: TName, options: CustomStringSelectOptions<TRequired, TDefault>) {
        return this.addOption<TName, "customStringSelect", TRequired, TDefault>(name, "customStringSelect", options)
    }

    addOptionCustomStringMultiSelect<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends string[] | undefined = undefined,
    >(name: TName, options: CustomStringMultiSelectOptions<TRequired, TDefault>) {
        return this.addOption<TName, "customStringMultiSelect", TRequired, TDefault>(name, "customStringMultiSelect", options)
    }

    addOptionCustomNumberSelect<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends number | undefined = undefined,
    >(name: TName, options: CustomNumberSelectOptions<TRequired, TDefault>) {
        return this.addOption<TName, "customNumberSelect", TRequired, TDefault>(name, "customNumberSelect", options)
    }

    addOptionCustomNumberMultiSelect<
        TName extends string,
        TRequired extends boolean = false,
        TDefault extends number[] | undefined = undefined,
    >(name: TName, options: CustomNumberMultiSelectOptions<TRequired, TDefault>) {
        return this.addOption<TName, "customNumberMultiSelect", TRequired, TDefault>(name, "customNumberMultiSelect", options)
    }

    private addOption<
        const TName extends string,
        const TKind extends OptionTypesKeys,
        const TRequired extends boolean = false,
        const TDefault extends InnerToConfigSetValueType<TKind> | undefined = undefined,
    >(name: TName, kind: TKind, options: OptionTypes<TRequired, TDefault>[TKind]) {
        this.definitions[name] = {
            kind: kind,
            ...(options as any),
        }

        return this as ListBuilder<LayerOption<TOpts, TName, TKind, TDefault, TRequired>>
    }

    complete(options?: ListOptions<OptionMapValues<TOpts>[]>): LoadedList<TOpts> {

        const def: OptionDefinitionList = {
            kind: "List",
            name: this.name,
            options: options ?? {},
            template: this.definitions,
        }

        this.manager.definitions.push(def)

        const value = this.manager.loadedSettings.find(v => v.name === this.name)
        return new LoadedList(def, value)
    }
}

function defToInternal(def: AnyOptionDefinition): Internal.SettingsOptionDefinition {
    if (def.kind === "Option") {
        return {
            kind: "Option",
            data: {
                name: def.name,
                label: def.definition.label ?? def.name,
                description: def.definition.description || "",
                required: def.definition.required ?? false,
                kind: optionTypesUnionToInternal(def.definition),
                defaultValue: def.definition.defaultValue ?? null,
            }
        }
    } else {
        return {
            kind: "List",
            data: {
                name: def.name,
                description: def.options.description || "",
                label: def.options.label ?? def.name,
                // asd: 10,
                required: false,
                defaultValue: def.options.defaultValue ?? null,
                template: Object.entries(def.template).map(([k, v]) => ({
                    name: k,
                    label: v.label ?? k,
                    defaultValue: v.defaultValue ?? null,
                    description: v.description ?? "",
                    required: v.required ?? false,
                    kind: optionTypesUnionToInternal(v),
                }))
            }
        }
    }
}

function optionTypesUnionToInternal(def: OptionTypesUnion): Internal.SettingsOptionType {
    switch (def.kind) {
        case "string":
            return {
                kind: "string",
                max_length: def.maxLength ?? null,
                min_length: def.minLength ?? null,
            }
        case "float":
            return {
                kind: "float",
                max: def.max ?? null,
                min: def.min ?? null,
            }
        case "integer":
            return {
                kind: "integer",
                max: def.max ?? null,
                min: def.min ?? null,
            }
        case "integer64":
            return {
                kind: "integer64",
                max: def.max ?? null,
                min: def.min ?? null,
            }
        case "boolean":
            return {
                kind: "boolean"
            }
        case "channel":
            return {
                kind: "channel",
                types: def.types ?? null,
            }
        case "channels":
            return {
                kind: "channels",
                types: def.types ?? null,
                max_length: def.max ?? null,
                min_length: 0
            }
        case "role":
            return {
                kind: "role",
                assignable: def.requireAssignable ?? null,
            }
        case "roles":
            return {
                kind: "roles",
                assignable: def.requireAssignable ?? null,
                max_length: def.max ?? null,
                min_length: 0
            }
        case "customStringSelect":
            return {
                kind: "customStringSelect",
                options: def.options,
            }
        case "customStringMultiSelect":
            return {
                kind: "customStringMultiSelect",
                options: def.options,
                max_selected: def.max_selected ?? null,
                min_selected: def.min_selected ?? null,
            }
        case "customNumberSelect":
            return {
                kind: "customNumberSelect",
                options: def.options,
            }
        case "customNumberMultiSelect":
            return {
                kind: "customNumberMultiSelect",
                options: def.options,
                max_selected: def.max_selected ?? null,
                min_selected: def.min_selected ?? null,
            }

    }
}

// I originally tried to use generics here to double check the return type 
// but i seem to run into weird typescript limitations regaring that
// and it also wouldn't be usable from the list then'
//
// This performs some basic protections and sanity checks, but more in depth ones may be implemented later
function loadOptionValue(value: any, definition: OptionTypesUnion): any {
    switch (definition.kind) {
        case "string": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "float": {
            return typeof value === "number" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "integer": {
            return typeof value === "number" ? Math.floor(value) : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "integer64": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "boolean": {
            if (typeof value === "string") {
                const lower = value.toLowerCase()
                if (lower === "yes" || lower === "true" || lower === "1") {
                    return true
                }

                return false
            }

            if (typeof value === "number") {
                if (value === 0) {
                    return false
                }

                return true
            }

            if (typeof value === "boolean") {
                return value
            }

            return getSysDefaultValue(definition.kind)
        }
        case "channel": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "channels": {
            return Array.isArray(value) ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "role": {
            return typeof value === "string" ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
        case "roles": {
            return Array.isArray(value) ? value : (definition.defaultValue ?? getSysDefaultValue(definition.kind))
        }
    }
}
