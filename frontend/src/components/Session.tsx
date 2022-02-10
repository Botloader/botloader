import React, { createContext, useContext, useEffect, useState } from "react";
import { ApiClient, isErrorResponse, User } from "botloader-common";
import { BuildConfig } from "../BuildConfig";
import { CreateFetcher } from "../Util";

export const SessionContext = createContext<Session>({
    apiClient: new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase),
});

export function SessionProvider({ children }: { children: React.ReactNode }) {
    let [session, setSession] = useState<Session>({
        apiClient: new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase),
    });

    useEffect(() => {
        async function validateAndUpdateSession(apiClient: ApiClient) {
            let user = await apiClient.getCurrentUser();
            if (!isErrorResponse(user)) {
                setSession({
                    user: user,
                    apiClient: apiClient,
                });
                console.log("session restored and valid: ", user);
            } else {
                localStorage.removeItem("botloader_token");
                console.log("session is invalid");
            }
        }

        async function restoreSession() {
            let token = localStorage.getItem("botloader_token");
            if (token) {
                // Got a token in storage, validate it and use it
                let client = new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase, token);
                await validateAndUpdateSession(client);
            } else {
                console.log("no token in local storage, no session to restore");
            }
        }

        restoreSession()
    }, []) // pass in [] to avoid running on each update

    return <SessionContext.Provider value={session}>{children}</SessionContext.Provider>
}

export function useSession(): Session {
    return useContext(SessionContext);
}

export interface Session {
    user?: User,
    apiClient: ApiClient,
}

export function RequireLoggedInSession({ children }: { children: React.ReactNode }) {
    let session = useSession();

    if (session.user) {
        return <React.Fragment> {children} </React.Fragment>
    }

    return <p>Not logged in or currently logging in...</p>
}