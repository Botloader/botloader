import React from "react"
import "./Panel.css"

type Props = {
    children?: React.ReactNode,
    title?: string,
}

export function Panel(props: Props) {
    return <div className="panel">
        {props.title ? <div className="panel-title"><p>{props.title}</p></div> : null}
        <div className="panel-body">
            {props.children}
        </div>
    </div>
}