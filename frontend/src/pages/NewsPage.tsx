import { isErrorResponse, NewsItem } from "botloader-common";
import { useEffect, useState } from "react"
import { useSession } from "../modules/session/useSession";
import { Loading } from "../components/Loading";
import { NewsItemComponent } from "../components/NewsItem";
import { Container, Stack } from "@mui/material";

export function NewsPage() {
    const session = useSession();
    const [news, setNews] = useState<undefined | null | NewsItem[]>(undefined);

    useEffect(() => {
        async function fetchNews() {
            let resp = await session.apiClient.getNews();
            if (isErrorResponse(resp)) {
                setNews(null);
            } else {
                setNews(resp);
            }
        }

        fetchNews();
    }, [session])

    return <Container>
        <Stack gap={5} marginTop={2}>
            {news === undefined ? <Loading />
                : news === null ? <p>Failed fetching news... :(</p>
                    : news.map(item_ => <NewsItemComponent key={item_.message_id} item={item_}></NewsItemComponent>)
            }
        </Stack>
    </Container>
}