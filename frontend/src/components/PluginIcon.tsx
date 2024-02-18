import { Avatar } from "@mui/material";
import { Plugin } from "botloader-common";
import { pluginImageUrl } from "../misc/pluginImageUrl";

export function PluginIcon({ plugin, size }: { plugin: Plugin, size?: "sm" | "m" }) {
    const iconImage = plugin.images.find(v => v.kind === "Icon")

    return <Avatar
        variant="rounded"
        sx={{
            width: size === "sm" ? 48 : 64,
            height: size === "sm" ? 48 : 64,
        }}
        src={iconImage ? pluginImageUrl(plugin.id, iconImage.image_id) : undefined}
    >
        {plugin.name[0] ?? "?"}
    </Avatar>
}