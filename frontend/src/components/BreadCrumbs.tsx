import { UIMatch, useMatches } from "react-router-dom";
import { OurRouteObject } from "../misc/ourRoute";
import { Breadcrumbs as MaterialBreadCrumbs, Typography } from "@mui/material";
import { LinkRouter } from "./LinkRouter";
import { useGuilds } from "../modules/guilds/GuildsProvider";
import { guildScriptsContext } from "../modules/guilds/GuildScriptsProvider";
import { useFetchedData } from "./FetchData";
import { pluginContext } from "./PluginProvider";
export function Breadcrumbs() {
    let matches = useMatches() as UIMatch<any, OurRouteObject["handle"]>[];
    let filteredMatches = matches
        // first get rid of any matches that don't have handle and crumb
        .filter((match) => Boolean(match.handle?.breadCrumb));

    const guilds = useGuilds()
    const scripts = useFetchedData(guildScriptsContext)
    const currentPlugin = useFetchedData(pluginContext)

    return (
        <MaterialBreadCrumbs aria-label="breadcrumb">
            {filteredMatches.map((match, index) => {

                const crumbElement = match.handle!.breadCrumb!(match.params, {
                    userGuilds: guilds?.all,
                    currentGuildScripts: scripts.value ?? undefined,
                    currentPlugin: currentPlugin.value ?? undefined,
                })

                if (match.handle?.breadCrumbCosmeticOnly) {
                    return <Typography
                        key={index}
                        color={index === filteredMatches.length - 1 ? "text.primary" : "inherit"}
                    >
                        {crumbElement}
                    </Typography>
                }

                return <LinkRouter
                    key={index}
                    color={index === filteredMatches.length - 1 ? "text.primary" : "inherit"}
                    to={match.pathname}
                    underline="hover"
                >

                    {crumbElement}
                </LinkRouter>
            })}

        </MaterialBreadCrumbs >
    );
}