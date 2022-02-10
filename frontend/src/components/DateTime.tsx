type Props = {
    dt: string,
}

export function DisplayDateTime(props: Props) {
    const parsed = new Date(props.dt);

    const date = parsed.toLocaleString();

    return <span className="datetime">{date}</span>
}