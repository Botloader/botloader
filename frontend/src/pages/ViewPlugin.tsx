import { Divider, Typography } from "@mui/material";
import { Container } from "@mui/system";
import ReactMarkdown from "react-markdown";
import { AddPluginToServerButton } from "../components/AddPluginToServer";
import { useFetchedDataBehindGuard } from "../components/FetchData";
import { pluginContext } from "../components/PluginProvider";

export function ViewPlugin() {
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);

    return <Container>
        <Typography>Plugin</Typography>
        <Typography variant="h4">{plugin.name}</Typography>
        <Divider />
        <Typography mb={2}>{plugin.short_description}</Typography>
        <AddPluginToServerButton />
        <ReactMarkdown>{plugin.long_description}</ReactMarkdown>
    </Container>
}