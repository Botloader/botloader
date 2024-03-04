import { Box, Checkbox, Chip, ListItemText, MenuItem, OutlinedInput, Select } from "@mui/material";
import { useCurrentFullGuild } from "../modules/guilds/FullGuildProvider";
import { Loading } from "./Loading";
import { useMemo } from "react";
import { DiscordChannel, DiscordRole } from "botloader-common";

type OptionsValue<TMultiple extends boolean> = TMultiple extends true ? string[] : string

export function SelectRole<
    TMultiple extends boolean,
>({ value, multiple, onChange, label, error }: {
    value: OptionsValue<TMultiple>,
    multiple: TMultiple,
    label: string
    error?: string | null
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
>({ value, multiple, onChange, label, error }: {
    value: OptionsValue<TMultiple>,
    multiple: TMultiple,
    label: string
    error?: string | null
    onChange: (value: OptionsValue<TMultiple>) => any,
}) {
    const guild = useCurrentFullGuild()

    const sortedChannels = useMemo(() => {
        if (!guild.value?.channels) {
            return []
        }

        let copy = [...guild.value.channels]
        copy.sort(
            (a, b) => (b.position ?? 0) - (a.position ?? 0)
        )
        return copy
    }, [guild.value?.channels])

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
                        <SelectedChannel key={value} allChannels={sortedChannels} channelId={value} />
                    ))
                    : <SelectedChannel allChannels={sortedChannels} channelId={selected} />

                }
            </Box>
        )}
        autoWidth={true}
    >
        {sortedChannels.map((channel) => (
            <MenuItem key={channel.id} value={channel.id}>
                {multiple && <Checkbox checked={value.includes(channel.id)} />}
                <ListItemText primary={channel.name} />
            </MenuItem>
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