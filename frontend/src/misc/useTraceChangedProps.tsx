import { useEffect, useRef } from "react";

export function useTraceChangedProps(props: any) {
    const prev = useRef(props);
    console.log(prev, props)
    useEffect(() => {
        const changedProps = Object.entries(props).reduce((ps, [k, v]) => {
            if (prev.current[k] !== v) {
                (ps as any)[k] = [prev.current[k], v];
            }
            return ps;
        }, {});
        if (Object.keys(changedProps).length > 0) {
            console.log('CHANGED PROPS:', changedProps);
        }
        prev.current = props;
    });
}
