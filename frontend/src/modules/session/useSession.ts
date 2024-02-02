import { useContext } from "react";
import { Session, SessionContext } from "./SessionContext";

export function useSession(): Session {
    return useContext(SessionContext);
}
