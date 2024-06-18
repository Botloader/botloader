import { Tooltip } from "@mui/material";

type Props = {
    dt: string,
}

export function DisplayDateTime(props: Props) {
    const parsed = new Date(props.dt);

    const date = parsed.toLocaleString();

    return <span className="datetime">{date}</span>
}

export function DisplayRelativeDateTime(props: Props) {
    const parsed = new Date(props.dt);
    const now = new Date()

    const deltaMs = now.getTime() - parsed.getTime()

    let descriptionBase = relativeStr(Math.abs(deltaMs))

    let fullDescription = ""
    if (descriptionBase !== undefined) {
        if (deltaMs > 0) {
            fullDescription = descriptionBase + " ago"
        } else {
            fullDescription = descriptionBase + " in the future"
        }
    } else {
        fullDescription = "just now"
    }

    const tooltip = parsed.toLocaleString();

    return <Tooltip title={tooltip}><span className="datetime">{fullDescription}</span></Tooltip>
}

function relativeStr(deltaMs: number) {

    const seconds = Math.floor(deltaMs / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)
    const weeks = Math.floor(days / 7)
    const months = Math.floor(days / 30)
    const years = Math.floor(days / 365)

    if (years > 0) {
        return years + " year" + pluralize(years)
    }

    if (months > 0) {
        return months + " month" + pluralize(months)
    }

    if (weeks > 0) {
        return weeks + " week" + pluralize(weeks)
    }

    if (days > 0) {
        return days + " day" + pluralize(days)
    }

    if (hours > 0) {
        return hours + " hour" + pluralize(hours)
    }

    if (minutes > 0) {
        return minutes + " minutes" + pluralize(minutes)
    }
}

function pluralize(num: number) {
    if (num !== 1) {
        return "s"
    } else {
        return ""
    }
}