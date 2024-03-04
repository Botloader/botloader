import { OpWrappers } from "./op_wrappers"
import * as Internal from "./generated/internal/index";

interface OptionTypes {
    string: {
        description?: string,
        required?: boolean,
        defaultValue?: string,
        maxLength?: number,
        minLength?: number,
    },
    float: {
        description?: string,
        required?: boolean,
        defaultValue?: number,
        max?: number,
        min?: number,
    },
    integer: {
        description?: string,
        required?: boolean,
        defaultValue?: number,
        max?: number,
        min?: number,
    },
    integer64: {
        description?: string,
        required?: boolean,
        defaultValue?: string,
        max?: string,
        min?: string,
    },
    channel: {
        description?: string,
        required?: boolean,
        defaultValue?: string,
    },
    channels: {
        description?: string,
        required?: boolean,
        defaultValue?: string[],
        min?: number,
        max?: number,
    },
    role: {
        description?: string,
        required?: boolean,
        defaultValue?: string,
    },
    roles: {
        description?: string,
        required?: boolean,
        defaultValue?: string[],
        min?: number,
        max?: number,
    },
}

type OptionTypesUnion = {
    [Key in keyof OptionTypes]: { kind: Key } & OptionTypes[Key]
}[keyof OptionTypes]

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

type InnerToConfigValue<T extends OptionTypesUnion> = ConfigOptionValueTypesMapping[T["kind"]]

type ToConfigValue<T extends OptionTypesUnion> =
    T["required"] extends true
    ? InnerToConfigValue<T>
    : T["defaultValue"] extends InnerToConfigValue<T>
    ? InnerToConfigValue<T>
    : InnerToConfigValue<T> | undefined

export type ToConfigValueMap<T extends Record<string, OptionTypesUnion>> = {
    [Key in keyof T]: ToConfigValue<T[Key]>
}

type ValidateOptions<TOptions, Base> = keyof TOptions extends keyof Base ? TOptions : never
type ValidateConfigMap<T extends Record<string, OptionTypesUnion>> = {
    [Key in keyof T]: ValidateOptions<T[Key], OptionTypes[T[Key]["kind"]] & { kind: string }>
}

type ValidateOptions3<T extends OptionTypesUnion> = ValidateOptions<T, OptionTypes[T["kind"]] & { kind: string }>

type OptionDefinition = {
    kind: "Option",
    name: string,
    description?: string,
    definition: OptionTypesUnion,
} | {
    kind: "List",
    name: string,
    description?: string,
    template: Record<string, OptionTypesUnion>,
    options: ListOptions<boolean, any>,
}


type ListOptions<TRequired extends boolean, TDefault> = {
    description?: string,
    max?: number,
    min?: number,
    required?: TRequired,
    defaultValue?: TDefault,
}

type OptionReturnType<TRequired, TDefault, TReturn> = TRequired extends true
    ? TReturn
    : TDefault | TReturn

export class SettingsManager {

    definitions: OptionDefinition[] = [];
    loadedSettings: [];

    constructor() {
        this.loadedSettings = OpWrappers.getSettings()
    }

    addOption<
        const T extends OptionTypesUnion,
    >(name: string, options: ValidateOptions3<T>): ToConfigValue<T> {

        this.definitions.push({
            kind: "Option",
            name,
            description: options.description,
            definition: options,
        })

        return null as any
        // throw new Error("TODO")
    }

    addOptionList<
        const T extends Record<string, OptionTypesUnion>,
        const TRequired extends boolean = false,
        const TDefault extends ToConfigValueMap<T>[] | undefined = undefined,
    >(
        name: string,
        options: ListOptions<TRequired, TDefault>,
        template: ValidateConfigMap<T>
    ): OptionReturnType<TRequired, TDefault, ToConfigValueMap<T>[]> {
        this.definitions.push({
            kind: "List",
            name,
            description: options.description,
            template: template,
            options: options,
        })

        const built = template;
        return built as any
    }

    /**
     * @internal
     */
    toInternalOptions(): Internal.SettingsOptionDefinition[] {
        return this.definitions.map(def => defToInternal(def))
    }
}

function defToInternal(def: OptionDefinition): Internal.SettingsOptionDefinition {
    if (def.kind === "Option") {
        return {
            kind: "Option",
            data: {
                name: def.name,
                description: def.description || "",
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
                min_length: def.min ?? null,
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
                min_length: def.min ?? null,
            }
    }
}