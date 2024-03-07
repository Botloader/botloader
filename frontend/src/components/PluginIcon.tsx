import { Avatar } from "@mui/material";
import { Plugin } from "botloader-common";
import { pluginImageUrl } from "../misc/pluginImageUrl";

const sizes = {
    xs: 32,
    sm: 48,
    m: 64,
}

export function PluginIcon({ plugin, size }: { plugin: Plugin, size?: keyof typeof sizes }) {
    const iconImage = plugin.images.find(v => v.kind === "Icon")

    return <Avatar
        variant="rounded"
        sx={{
            width: sizes[size ?? "m"],
            height: sizes[size ?? "m"],
        }}
        src={iconImage ? pluginImageUrl(plugin.id, iconImage.image_id) : undefined}
    >
        {plugin.name[0] ?? "?"}
    </Avatar>
}