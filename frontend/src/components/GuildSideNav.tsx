import { Divider, List, ListItem, ListItemText } from "@mui/material";
import { useCurrentGuild } from "../modules/guilds/CurrentGuild";
import { useCurrentGuildScripts } from "../modules/guilds/GuildScriptsProvider";
import { NavItem, SideNav } from "./SideNav";
import { Loading } from "./Loading";

export function GuildSideNav() {
    const guild = useCurrentGuild();
    const { value: data } = useCurrentGuildScripts()

    const navItems = [
        {
            label: "Home",
            isNavLink: true,
            exact: true,
            path: `/servers/${guild?.value?.guild.id}`,
        },
    ]


    const pluginScripts = data?.scripts.filter(v => Boolean(v.plugin_id))
    // if (!pluginScripts) {
    //     navItems.push("loading" as any)
    // } else {
    //     for (const script of pluginScripts) {
    //         navItems.push({
    //             label: script.name,
    //             path: `/servers/${guild?.value?.guild.id}/scripts/${script.id}`,
    //             exact: true,
    //             isNavLink: true,
    //         })
    //     }
    // }

    return <SideNav items={navItems}>
        <Divider />
        <ListItem>
            <ListItemText>Plugins</ListItemText>
        </ListItem>
        <Divider />
        {!pluginScripts && <Loading />}
        {pluginScripts?.map(script => (<NavItem key={script.id} item={{
            label: script.name,
            path: `/servers/${guild?.value?.guild.id}/scripts/${script.id}`,
            exact: true,
            isNavLink: true,
        }} />))}
    </SideNav>
}
