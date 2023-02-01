import React from "react";

export function Gist({id, file}:{id: string, file?: string}){
    const fileArg = file ? `?file=${file}` : "";

    return <iframe title={`gist-${id}`}>
        <script src={`https://gist.github.com/${id}.js${fileArg}`}></script>
    </iframe>
}