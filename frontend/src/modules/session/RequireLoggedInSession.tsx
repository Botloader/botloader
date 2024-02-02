import { Navigate } from "react-router-dom";
import { useSession } from "./useSession";

export function RequireLoggedInSession({ children }: { children: React.ReactNode }) {
    const session = useSession();

    if (!session.initialized) {
        return <p>Loading...</p>
    }

    if (session.user) {
        return <>{children}</>
    } else if (session.signingIn) {
        return <p>Logging you in...</p>
    } else {
        return <Navigate to="/"></Navigate>
    }
}