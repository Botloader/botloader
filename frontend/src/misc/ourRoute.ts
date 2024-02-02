import { BotGuild, Plugin, ScriptPluginData, ScriptsWithPlugins } from "botloader-common";
import { IndexRouteObject, NonIndexRouteObject } from "react-router-dom";

type OurIndexRouteObject = Omit<IndexRouteObject, "handle" | "children">
type OurNonIndexRouteObject = Omit<NonIndexRouteObject, "handle" | "children">


export type OurRouteObject = (OurIndexRouteObject | OurNonIndexRouteObject & {
    children?: OurRouteObject[]
}) & {
    handle?: RouteHandle
}

export type RouteHandle = {
    breadCrumb?: (params: BreadCrumbParams, data: BreadCrumbData) => string,
    breadCrumbCosmeticOnly?: boolean,
}

export type BreadCrumbParams = {
    guildId?: string,
    pluginId?: string,
    scriptId?: string,
}

export type BreadCrumbData = {
    userGuilds?: BotGuild[],
    currentGuildScripts?: ScriptsWithPlugins,
    currentPlugin?: Plugin<ScriptPluginData>
}