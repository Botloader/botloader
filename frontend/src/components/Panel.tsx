import React from "react"
import "./Panel.css"

type Props = {
    children?: React.ReactNode,
    title?: string,
    className?: string,
}

export function Panel(props: Props) {
    return <div className={props.className ? "panel " + props.className : "panel"}>
        {props.title ? <h3 className="panel-title">{props.title}</h3> : null}
        <div className="panel-body">
            {props.children}
        </div>
    </div>
}