import { OpWrappers } from "./op_wrappers"
import * as Internal from "./generated/internal/index";
import { ChannelType } from "./discord/index";

interface BaseOptions<TRequired, TDefault> {
    label?: string
    description?: string
    required?: TRequired
    defaultValue?: TDefault,
}

interface StringOptions<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    maxLength?: number,
    minLength?: number,
}

interface FloatOptions<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    max?: number,
    min?: number,
}

interface IntegerOptions<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    max?: number,
    min?: number,
}

interface Integer64Options<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    max?: string,
    min?: string,
}

interface ChannelOptions<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    types?: ChannelType[]
}

interface ChannelsOptions<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    allowEmpty?: boolean,
    types?: ChannelType[]
    max?: number,
}

interface RoleOptions<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    requireAssignable?: boolean
}

interface RolesOptions<TRequired, TDefault> extends BaseOptions<TRequired, TDefault> {
    allowEmpty?: boolean,
    max?: number,
}

interface OptionTypes<TRequired, TDefault> {
    string: StringOptions<TRequired, TDefault>,
    float: FloatOptions<TRequired, TDefault>,
    integer: IntegerOptions<TRequired, TDefault>,
    integer64: Integer64Options<TRequired, TDefault>,
    channel: ChannelOptions<TRequired, TDefault>,
    channels: ChannelsOptions<TRequired, TDefault>,
    role: RoleOptions<TRequired, TDefault>,
    roles: RolesOptions<TRequired, TDefault>,
}

type OptionTypesKeys = keyof OptionTypes<boolean, any>

type OptionTypesUnion = {
    [Key in OptionTypesKeys]: { kind: Key } & OptionTypes<boolean, any>[Key]
}[OptionTypesKeys]

export interface ConfigOptionValueTypesMapping {
    string: string,
    integer: number,
    integer64: string,
    float: number,
    channel: string,
    channels: string[],
    role: string,
    roles: string[],
}

type InnerToConfigValue<TKind extends OptionTypesKeys> = ConfigOptionValueTypesMapping[TKind]

type ToConfigValue<TKind extends OptionTypesKeys, TDefaultValue, TRequired extends boolean> =
    TRequired extends true
    ? InnerToConfigValue<TKind>
    : InnerToConfigValue<TKind> | TDefaultValue
// ? InnerToConfigValue<TKind>
// : InnerToConfigValue<TKind> | undefined

type OptionDefinitionOption = {
    kind: "Option",
    name: string,
    definition: OptionTypesUnion,
}

type OptionDefinitionList = {
    kind: "List",
    name: string,
    label?: string,
    description?: string,
    template: Record<string, OptionTypesUnion>,
    options: ListOptions<boolean, any>,
}

type AnyOptionDefinition = OptionDefinitionOption | OptionDefinitionList

type ListOptions<TRequired extends boolean, TDefault> = {
    description?: string,
    max?: number,
    min?: number,
    required?: TRequired,
    defaultValue?: TDefault,
}

export class LoadedOption<TKind extends OptionTypesKeys, TDefaultValue, TRequired extends boolean> {
    definition: OptionDefinitionOption
    value: ToConfigValue<TKind, TDefaultValue, TRequired>

    /**
     * @internal
     */
    constructor(definition: OptionDefinitionOption, rawValue?: Internal.SettingsOptionValue) {
        this.definition = definition

        if (rawValue) {
            this.value = loadOptionValue(rawValue.value, definition.definition)
        } else {
            this.value = definition.definition.defaultValue
        }
    }
}

export class SettingsManager {

    definitions: AnyOptionDefinition[] = [];
    loadedSettings: Internal.SettingsOptionValue[];

    constructor(scriptId: number) {
        this.loadedSettings = OpWrappers.getSettings(scriptId)
    }

    addOptionString<
        TDefault extends string | undefined = undefined,
    >(name: string, options: StringOptions<false, TDefault>) {
        return this.addOption<"string", false, TDefault>(name, "string", options)
    }

    addOptionFloat<
        TDefault extends number | undefined = undefined,
    >(name: string, options: FloatOptions<false, TDefault>) {
        return this.addOption<"float", false, TDefault>(name, "float", options)
    }

    addOptionInteger<
        TDefault extends number | undefined = undefined,
    >(name: string, options: IntegerOptions<false, TDefault>) {
        return this.addOption<"integer", false, TDefault>(name, "integer", options)
    }

    addOptionInteger64<
        TDefault extends string | undefined = undefined,
    >(name: string, options: Integer64Options<false, TDefault>) {
        return this.addOption<"integer64", false, TDefault>(name, "integer64", options)
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

    private addOption<
        const TKind extends OptionTypesKeys,
        const TRequired extends boolean = false,
        const TDefault extends InnerToConfigValue<TKind> | undefined = undefined,
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

    startList(name: string) {
        return new ListBuilder<{}>(this)
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

type LayerOption<TInner, TKey extends string, TKind extends keyof ConfigOptionValueTypesMapping, TDefault, TRequired> =
    { [Prop in keyof TInner]: TInner[Prop] } & { [Prop in TKey]: { defaultValue: TDefault, kind: TKind, required: TRequired } };

type OptionMapValues<T extends OptionsMap> = {
    [Prop in keyof T]: ToConfigValue<T[Prop]["kind"], T[Prop]["defaultValue"], T[Prop]["required"]>
}

// interface LoadedList<TOpts extends OptionsMap> {
//     definitions: AnyOptionDefinition[],
//     value: OptionMapValues<TOpts>[]
// }

export class LoadedList<TOpts extends OptionsMap> {
    definition: OptionDefinitionList
    value: OptionMapValues<TOpts>[]

    /**
     * @internal
     */
    constructor(definition: OptionDefinitionList, rawValue?: Internal.SettingsOptionValue) {
        this.definition = definition

        if (!rawValue?.value) {
            this.value = definition.options.defaultValue
            return
        }

        const arrValue = rawValue?.value
        if (!Array.isArray(arrValue)) {
            this.value = definition.options.defaultValue
            return
        }

        this.value = []

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
                    } else {
                        obj[name] = optionDefinition.defaultValue
                    }
                }
            }

            this.value.push(obj)
        }
    }
}

export class ListBuilder<TOpts extends OptionsMap> {
    manager: SettingsManager;
    definitions: OptionDefinitionOption[] = [];

    constructor(manager: SettingsManager) {
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

    private addOption<
        const TName extends string,
        const TKind extends OptionTypesKeys,
        const TRequired extends boolean = false,
        const TDefault extends InnerToConfigValue<TKind> | undefined = undefined,
    >(name: TName, kind: TKind, options: OptionTypes<TRequired, TDefault>[TKind]) {
        this.definitions.push({
            kind: "Option",
            name,
            definition: { kind, ...options } as any,
        })

        return this as ListBuilder<LayerOption<TOpts, TName, TKind, TDefault, TRequired>>
    }

    complete(): LoadedList<TOpts> {

        return null as any
    }
}

function defToInternal(def: AnyOptionDefinition): Internal.SettingsOptionDefinition {
    if (def.kind === "Option") {
        return {
            kind: "Option",
            data: {
                name: def.name,
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
                description: def.description || "",
                required: def.options.required ?? false,
                defaultValue: def.options.defaultValue ?? null,
                template: Object.entries(def.template).map(([k, v]) => ({
                    name: k,
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
                max: null,
                min: null,
            }
        case "integer64":
            return {
                kind: "integer64",
                max: def.max ?? null,
                min: def.min ?? null,
            }
        case "channel":
            return {
                kind: "channel",
                types: null,
            }
        case "channels":
            return {
                kind: "channels",
                types: null,
                max_length: def.max ?? null,
                min_length: def.allowEmpty ? 0 : 1,
            }
        case "role":
            return {
                kind: "role",
                assignable: null
            }
        case "roles":
            return {
                kind: "roles",
                assignable: null,
                max_length: def.max ?? null,
                min_length: def.allowEmpty ? 0 : 1,
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
            return typeof value === "string" ? value : definition.defaultValue
        }
        case "float": {
            return typeof value === "number" ? value : definition.defaultValue
        }
        case "integer": {
            return typeof value === "number" ? Math.floor(value) : definition.defaultValue
        }
        case "integer64": {
            return typeof value === "string" ? value : definition.defaultValue
        }
        case "channel": {
            return typeof value === "string" ? value : definition.defaultValue
        }
        case "channels": {
            return Array.isArray(value) ? value : definition.defaultValue
        }
        case "role": {
            return typeof value === "string" ? value : definition.defaultValue
        }
        case "roles": {
            return Array.isArray(value) ? value : definition.defaultValue
        }
    }
}