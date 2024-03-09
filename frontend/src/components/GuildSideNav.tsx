import { Box, Divider, List, ListItem, ListItemText } from "@mui/material";
import { useCurrentGuild } from "../modules/guilds/CurrentGuild";
import { useCurrentGuildScripts } from "../modules/guilds/GuildScriptsProvider";
import { NavItem, SideNav } from "./SideNav";
import { Loading } from "./Loading";
import { PluginIcon } from "./PluginIcon";

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

    const guildScripts = data?.scripts.filter(v => !v.plugin_id)
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
            <ListItemText
                primaryTypographyProps={{
                    variant: "overline",
                    color: "grey"
                }}
            >Plugins</ListItemText>
        </ListItem>
        {!pluginScripts && <Loading />}
        <List>

            {pluginScripts?.map(script => (<NavItem
                key={script.id}
                item={{
                    label: script.name,
                    path: `/servers/${guild?.value?.guild.id}/scripts/${script.id}`,
                    exact: true,
                    isNavLink: true,
                }}
                icon={<PluginIcon
                    plugin={data?.plugins.find(v => v.id === script.plugin_id)!}
                    size="xs"
                />}

                indicator={<ScriptEnabledIndicator enabled={script.enabled} />}

            // icon={<MailIcon />}
            />))}
        </List>
        <Divider />
        <List>

            <ListItem>
                <ListItemText
                    primaryTypographyProps={{
                        variant: "overline",
                        color: "grey"
                    }}
                >Scripts</ListItemText>
            </ListItem>
            {guildScripts?.map(script => (<NavItem
                key={script.id}
                item={{
                    label: script.name,
                    path: `/servers/${guild?.value?.guild.id}/scripts/${script.id}`,
                    exact: true,
                    isNavLink: true,
                }}
                indicator={<ScriptEnabledIndicator enabled={script.enabled} />}
            />))}


        </List>
    </SideNav>
}

function ScriptEnabledIndicator({ enabled }: { enabled: boolean }) {
    return <Box
        width={"8px"}
        height={"8px"}
        borderRadius={"4px"}
        sx={{
            backgroundColor: enabled ? "#31c631" : "#ff3939"
        }}
    ></Box>
} 