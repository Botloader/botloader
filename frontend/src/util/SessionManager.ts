import { ApiClient, isErrorResponse } from "botloader-common";
import { BuildConfig } from "../BuildConfig";
import { Session } from "../components/Session";
import { CreateFetcher } from "../Util";

const anonApiClient = new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase);

class SessionManager {
    private _session: Session = {
        apiClient: anonApiClient,
        signingIn: true,
    };

    lastEventHandlerId: number = 0;

    eventHandlers: EventHandlers = {};

    get session() {
        return this._session;
    }

    set session(session: Session) {
        this._session = session;
        for (let [, value] of Object.entries(this.eventHandlers)) {
            (value as EventHandler)(session);
        }
    }

    constructor() {
        this.restoreSession();
    }

    private async validateAndUpdateSession(apiClient: ApiClient) {
        let user = await apiClient.getCurrentUser();
        if (!isErrorResponse(user)) {
            this.session = {
                user: user,
                apiClient: apiClient,
                signingIn: false,
            };
            console.log("session restored and valid: ", user);
        } else {
            localStorage.removeItem("botloader_token");
            this.session = {
                apiClient: anonApiClient,
                signingIn: false,
            };
            console.log("session is invalid");

        }
    }

    private async restoreSession() {
        let token = localStorage.getItem("botloader_token");
        if (token) {
            // Got a token in storage, validate it and use it
            let client = new ApiClient(CreateFetcher(), BuildConfig.botloaderApiBase, token);
            await this.validateAndUpdateSession(client);
        } else {
            console.log("no token in local storage, no session to restore");
            this.session = {
                apiClient: anonApiClient,
                signingIn: false,
            };
        }
    }

    async logout() {
        await this.session.apiClient.logout();
        this.session = {
            apiClient: anonApiClient,
            signingIn: false,
        };
    }

    async logoutAllSessions() {
        await this.session.apiClient.deleteAllSessions();
        this.session = {
            apiClient: anonApiClient,
            signingIn: false,
        };
    }

    subscribe(handler: EventHandler): number {
        const id = this.lastEventHandlerId++;
        this.eventHandlers[id] = handler;

        return id;
    }

    unSubscribe(id: number) {
        delete this.eventHandlers[id];
    }
}

export const sessionManager = new SessionManager();

interface EventHandlers {
    [key: number]: EventHandler,
}

type EventHandler = (evt: Session) => any;