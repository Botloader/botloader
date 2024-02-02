import React, { createContext, useEffect, useState } from "react";
import { ApiClient, User, isErrorResponse } from "botloader-common";
import { CreateFetcher } from "../../Util";
import { BuildConfig } from "../../BuildConfig";

const anonApiClient = new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase);

export type Session = {
    user?: User,
    apiClient: ApiClient,
    signingIn: boolean,
    initialized: boolean
    logOut: () => any,
    logOutAllUserSessions: () => any,
}

const defaultSession: Session = {
    apiClient: new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase),
    signingIn: false,
    initialized: false,
    logOut: () => { },
    logOutAllUserSessions() { },
}

export const SessionContext = createContext<Session>(defaultSession);

export function SessionProvider({ children }: { children: React.ReactNode }) {
    let [sessionData, setSessionData] = useState<SessionStateData>({
        apiClient: new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase),
        signingIn: false,
        initialized: false,
    });

    async function validateAndUpdateSession(apiClient: ApiClient) {
        let user = await apiClient.getCurrentUser();
        if (!isErrorResponse(user)) {

            setSessionData({
                user: user,
                apiClient: apiClient,
                signingIn: false,
                initialized: true,
            });

            console.log("session restored and valid: ", user);
        } else {
            localStorage.removeItem("botloader_token");
            setSessionData({
                apiClient: anonApiClient,
                signingIn: false,
                initialized: true,
            });
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
            setSessionData({
                apiClient: anonApiClient,
                signingIn: false,
                initialized: true,
            });
        }
    }

    async function logout() {
        await sessionData.apiClient.logout();
        setSessionData({
            apiClient: anonApiClient,
            signingIn: false,
            initialized: true,
        })
    }

    async function logoutAllSessions() {
        await sessionData.apiClient.deleteAllSessions();
        setSessionData({
            apiClient: anonApiClient,
            signingIn: false,
            initialized: true,
        })
    }

    useEffect(() => {
        restoreSession()
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [])

    const session: Session = {
        apiClient: sessionData.apiClient,
        initialized: sessionData.initialized,
        signingIn: false,
        user: sessionData.user,
        logOut: logout,
        logOutAllUserSessions: logoutAllSessions,
    }

    if (session.initialized) {
    }

    return <SessionContext.Provider value={session}>{children}</SessionContext.Provider>
}

type SessionStateData = {
    user?: User,
    apiClient: ApiClient,
    signingIn: boolean,
    initialized: boolean,
}