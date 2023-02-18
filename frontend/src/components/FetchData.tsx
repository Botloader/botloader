import { ApiResult, isErrorResponse } from "botloader-common";
import { createContext, ReactNode, useCallback, useContext, useEffect } from "react";
import { useState } from "react";

// the returned data from the hook
//
// IsBehindGuard can be set as a convenience if you can be sure
// that the resource is loaded (if for example it is behind a guard
// that will only render children if its loaded)
// removing null and undefined from the value
export type FetchDataHook<
    T,
    IsBehindGuard extends boolean = false
> = IsBehindGuard extends true
    ? FetchDataHookBehindGuard<T>
    : FetchDataHookNotBehindGuard<T>;

export type ResolvedApiResult<T> = Awaited<T> extends ApiResult<infer U> ? U : T;

export interface FetchDataHookNotBehindGuard<T> {
    value?: T | null;
    error?: Error | null;
    loading: boolean;
    reload: () => void;
    setData: (d: SetData<T>) => void;
}

export interface FetchDataHookBehindGuard<T> {
    value: T;
    loading: false;
    reload: () => void;
    setData: (d: SetData<T>) => void;
}

// either a new value or a function that returns a new value while getting passed the current one
export type SetData<T> = T | ((cur: T | null) => T);

// This Component/Context provider loads a remote resource returning the current load state with
// methods to update and reload the resource
export function useFetchData<T, U extends ResolvedApiResult<T> = ResolvedApiResult<T>>(
    loader: () => Promise<T>
): FetchDataHook<U> {

    let [result, setResult] = useState<{ state: "loading" | "loaded", value: U | null, error: Error | null }>({
        state: "loading",
        value: null,
        error: null
    });

    const load = useCallback(async () => {
        setResult({ state: "loading", value: null, error: null });

        try {
            let resp = await loader();
            if (isErrorResponse(resp)) {
                setResult({
                    state: "loaded",
                    value: null,
                    error: new Error(`Operation failed: ${resp.resp_code}: ${resp.response?.description} ${resp.response?.code}`)
                });
            } else {
                setResult({
                    state: "loaded",
                    value: resp as U,
                    error: null,
                })
            }
        } catch (error) {
            setResult({
                state: "loaded",
                value: null,
                error: error as Error,
            })
        }
    }, [loader]);

    useEffect(() => {
        load();
    }, [load]);

    return {
        reload: load,
        value: result.value,
        error: result.error,
        loading: result.state === "loading",
        setData: (val) => {
            if (typeof val === "function") {
                setResult((current) => {
                    return {
                        error: null,
                        state: "loaded",
                        value: (val as (v: U | null) => U)(current.value),
                    }
                })
            } else {
                setResult({
                    state: "loaded",
                    value: val,
                    error: null,
                });
            }
        },
    };
}

export function FetchData<T>({ loader, context, children }: {
    loader: () => Promise<T>,
    context: React.Context<FetchDataHook<ResolvedApiResult<T>>>,
    children: ReactNode
}) {
    let hook = useFetchData(loader);

    return <context.Provider value={hook}>
        <FetchDataGuard context={context}>
            {children}
        </FetchDataGuard>
    </context.Provider>
}

export function useFetchedData<T, Guarded extends boolean = false>(context: React.Context<FetchDataHook<T, Guarded>>): FetchDataHook<T, Guarded> {
    return useContext(context)
}

export function useFetchedDataBehindGuard<T>(context: React.Context<FetchDataHook<T>>): FetchDataHook<T, true> {
    return useContext(context) as FetchDataHook<T, true>
}

export function FetchDataGuard<T>(props: { context: React.Context<FetchDataHook<T>>, children: ReactNode }) {
    let hook = useFetchedData(props.context);
    if (hook.loading) {
        return <p>Loading</p>
    }

    if (hook.value) {
        return <>{props.children}</>
    }

    return <p>Something went wrong: {JSON.stringify(hook.error)}</p>
}

export function createFetchDataContext<T>(): React.Context<FetchDataHook<T>> {
    let defaultHook: FetchDataHook<T> = {
        loading: true,
        reload: () => { },
        setData: (d: SetData<T>) => { },
        error: null,
        value: null,
    };

    return createContext(defaultHook);
}