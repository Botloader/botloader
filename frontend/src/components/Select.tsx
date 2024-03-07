import { Box, Checkbox, Chip, ListItemText, MenuItem, OutlinedInput, Select } from "@mui/material";
import { useCurrentFullGuild } from "../modules/guilds/FullGuildProvider";
import { Loading } from "./Loading";
import { useMemo } from "react";
import { ChannelType, DiscordChannel, DiscordNumberedChannelTypes, DiscordRole } from "botloader-common";

type OptionsValue<TMultiple extends boolean> = TMultiple extends true ? string[] : string

export function SelectRole<
    TMultiple extends boolean,
>({ value, multiple, onChange, label, error, allowEmpty }: {
    value: OptionsValue<TMultiple>,
    multiple: TMultiple,
    label: string
    error?: string | null,
    allowEmpty?: boolean,
    onChange: (value: OptionsValue<TMultiple>) => any,
}) {
    const guild = useCurrentFullGuild()

    const sortedRoles = useMemo(() => {
        if (!guild.value?.roles) {
            return []
        }

        let copy = [...guild.value.roles]
        copy.sort(
            (a, b) => b.position - a.position
        )

        return copy
    }, [guild.value?.roles])

    if (!guild.value) {
        return <Loading />
    }

    console.log(guild.value.channels)

    return <Select
        multiple={multiple}
        value={value}
        onChange={(evt) => {
            onChange(evt.target.value as any)
        }}
        input={<OutlinedInput label={label} />}
        SelectDisplayProps={{
            // style: {
            //     padding:  12,
            // }
        }}
        error={Boolean(error)}
        renderValue={(selected) => (
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, marginRight: 2 }}>
                {Array.isArray(selected)
                    ? selected.map((value) => (
                        <SelectedRole key={value} allRoles={sortedRoles} roleId={value} />
                    ))
                    : <SelectedRole allRoles={sortedRoles} roleId={selected} />

                }
            </Box>
        )}
        autoWidth={true}
    >
        {!multiple && allowEmpty && <MenuItem value={""}>
            <ListItemText primary={"None"} />
        </MenuItem>}

        {sortedRoles.map((role) => (
            <MenuItem key={role.id} value={role.id}>
                {multiple && <Checkbox checked={value.includes(role.id)} />}
                <ListItemText primary={role.name} />
                <RoleColorDot role={role} size={12} />
            </MenuItem>
        ))}
    </Select>
}

function SelectedRole({ allRoles, roleId }: { allRoles: DiscordRole[], roleId: string }) {
    const role = allRoles.find(v => v.id === roleId)

    return <Chip
        size="small"
        icon={<Box>{role && <RoleColorDot size={12} role={role} />}</Box>}
        label={role?.name ?? roleId}
    />
}

function RoleColorDot({ size, role }: { size: number, role: DiscordRole }) {
    if (role.color === 0) {
        return <></>
    }

    const colorHex = role.color.toString(16)

    return <Box
        sx={{
            backgroundColor: "#" + colorHex,
            minHeight: size + "px",
            minWidth: size + "px",
            borderRadius: "5px",
            // paddingLeft: "5px"
        }} />
}


export function SelectChannel<
    TMultiple extends boolean,
>({ value, multiple, onChange, label, error, allowEmpty, types }: {
    value: OptionsValue<TMultiple>,
    multiple: TMultiple,
    label: string
    error?: string | null
    allowEmpty?: boolean
    types?: ChannelType[]
    onChange: (value: OptionsValue<TMultiple>) => any,
}) {
    const guild = useCurrentFullGuild()

    const sortedChannels = useMemo(() => {
        if (!guild.value?.channels) {
            return []
        }

        const filterBy = types?.map(v => ChannelTypeToNumber(v))

        // const copy = 

        let copy = guild.value.channels.filter(v =>
            !filterBy || filterBy.includes(v.type))
        copy.sort(
            (a, b) => (b.position ?? 0) - (a.position ?? 0)
        )

        const categorySorted: {
            name: string,
            id?: string,
            position?: number,
            children: DiscordChannel[]
        }[] = [{
            name: "",
            children: []
        }]

        for (const item of guild.value.channels) {
            if (item.type !== DiscordNumberedChannelTypes.GuildCategory) {
                continue
            }

            categorySorted.push({
                name: item.name ?? "",
                children: [],
                id: item.id,
                position: item.position
            })
        }

        categorySorted.sort((a, b) =>
            (a.position ?? 0) - (b.position ?? 0))

        for (const item of copy) {
            if (item.type === DiscordNumberedChannelTypes.GuildCategory) {
                continue
            }

            let parentCategory;
            if (item.parent_id) {
                parentCategory = categorySorted.find(v => v.id === item.parent_id)
            }

            if (parentCategory) {
                parentCategory.children.push(item)
            } else {
                categorySorted[0].children.push(item)
            }
        }

        return categorySorted
    }, [guild.value?.channels, types])

    if (!guild.value) {
        return <Loading />
    }

    return <Select
        multiple={multiple}
        value={value}
        onChange={(evt) => {
            onChange(evt.target.value as any)
        }}
        input={<OutlinedInput label={label} />}
        SelectDisplayProps={{
            // style: {
            //     padding:  12,
            // }
        }}

        error={Boolean(error)}

        renderValue={(selected) => (
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, marginRight: 2 }}>
                {Array.isArray(selected)
                    ? selected.map((value) => (
                        <SelectedChannel key={value} allChannels={guild.value?.channels ?? []} channelId={value} />
                    ))
                    : <SelectedChannel allChannels={guild.value?.channels ?? []} channelId={selected} />

                }
            </Box>
        )}
        autoWidth={true}
    >
        {!multiple && allowEmpty && <MenuItem value={""}>
            <ListItemText primary={"None"} />
        </MenuItem>}

        {sortedChannels.map((group) => (
            [
                group.id && (
                    (!types || types.includes("Category")) || group.children.length > 0
                ) && <MenuItem
                    key={group.id}
                    value={group.id}
                    disabled={types && !types.includes("Category")}
                >
                    {multiple && <Checkbox checked={value.includes(group.id)} />}
                    <ListItemText primary={group.name} />
                </MenuItem>,

                ...group.children.map(channel => (
                    <MenuItem
                        key={channel.id}
                        value={channel.id}
                        sx={{
                            paddingLeft: channel.parent_id ? 5 : 2,
                        }}
                    >
                        {multiple && <Checkbox checked={value.includes(channel.id)} />}
                        <ListItemText primary={channel.name} />
                    </MenuItem>
                ))
            ]
        ))}
    </Select>
}

function SelectedChannel({ allChannels, channelId }: { allChannels: DiscordChannel[], channelId: string }) {
    const channel = allChannels.find(v => v.id === channelId)

    return <Chip
        size="small"
        label={channel?.name ?? channelId}
    />
}

function ChannelTypeToNumber(inputKind: ChannelType): DiscordNumberedChannelTypes {
    switch (inputKind) {
        case "Text":
            return DiscordNumberedChannelTypes.GuildText

        case "Voice":
            return DiscordNumberedChannelTypes.GuildVoice

        case "Category":
            return DiscordNumberedChannelTypes.GuildCategory

        case "News":
            return DiscordNumberedChannelTypes.GuildAnnouncement

        case "NewsThread":
            return DiscordNumberedChannelTypes.AnnouncementThread

        case "PublicThread":
            return DiscordNumberedChannelTypes.PublicThread

        case "PrivateThread":
            return DiscordNumberedChannelTypes.PrivateThread

        case "StageVoice":
            return DiscordNumberedChannelTypes.GuildStageVoice

        case "GuildDirectory":
            return DiscordNumberedChannelTypes.GuildDirectory

        case "Forum":
            return DiscordNumberedChannelTypes.GuildForum
    }

    throw new Error("unsupported channel type: " + inputKind)
}