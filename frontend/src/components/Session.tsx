import React, { createContext, useContext, useEffect, useState } from "react";
import { ApiClient, User } from "botloader-common";
import { BuildConfig } from "../BuildConfig";
import { CreateFetcher } from "../Util";
import { sessionManager } from "../util/SessionManager";
import { Navigate } from "react-router-dom";

export const SessionContext = createContext<Session>({
    apiClient: new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase),
    signingIn: false,
});

export function SessionProvider({ children }: { children: React.ReactNode }) {
    let [session, setSession] = useState<Session>(sessionManager.session);

    useEffect(() => {
        setSession(sessionManager.session);
        let handlerId = sessionManager.subscribe((newSession) => setSession(newSession));
        return () => {
            sessionManager.unSubscribe(handlerId);
        }
    }, [])

    return <SessionContext.Provider value={session}>{children}</SessionContext.Provider>
}

export function useSession(): Session {
    return useContext(SessionContext);
}

export interface Session {
    user?: User,
    apiClient: ApiClient,
    signingIn: boolean,
}

export function RequireLoggedInSession({ children }: { children: React.ReactNode }) {
    const session = useSession();

    if (session.user) {
        return <React.Fragment> {children} </React.Fragment>
    } else if (session.signingIn) {
        return <p>Logging you in...</p>
    } else {
        return <Navigate to="/"></Navigate>
    }
}