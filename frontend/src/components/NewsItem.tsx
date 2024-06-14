import { NewsItem } from "botloader-common";
import ReactMarkdown from "react-markdown";
import { Card, CardContent, CardHeader, Typography } from "@mui/material";

export function NewsItemComponent(props: { item: NewsItem }) {
    return <Card sx={{ width: "100%" }}>
        <CardHeader
            title={"#" + props.item.channel_name}
            subheader={new Date(props.item.posted_at).toLocaleString()}
        />
        <CardContent>
            <Typography >
                <ReactMarkdown>{props.item.content}</ReactMarkdown>
            </Typography>
        </CardContent>
    </Card>
}