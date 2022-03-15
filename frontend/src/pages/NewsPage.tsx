import { isErrorResponse, NewsItem } from "botloader-common";
import { useEffect, useState } from "react"
import { Panel } from "../components/Panel";
import { useSession } from "../components/Session";
import ReactMarkdown from 'react-markdown'

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

    return <>
        {news === undefined ? <p>Loading...</p>
            : news === null ? <p>Failed fetching news... :(</p>
                : news.map(item_ => <NewsItemComponent key={item_.message_id} item={item_}></NewsItemComponent>)
        }
    </>
}

function NewsItemComponent(props: { item: NewsItem }) {
    return <Panel>
        <div className="news-item">
            <h3>#{props.item.channel_name} - {new Date(props.item.posted_at).toLocaleString()}</h3>
            <ReactMarkdown>{props.item.content}</ReactMarkdown>
            {/* <p>{marked.parse(props.item.content)}</p> */}
        </div>
    </Panel>
}