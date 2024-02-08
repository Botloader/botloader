import { Navigate } from "react-router-dom";
import { useSession } from "./useSession";
import { Loading } from "../../components/Loading";

export function RequireLoggedInSession({ children }: { children: React.ReactNode }) {
    const session = useSession();

    if (!session.initialized) {
        return <Loading />
    }

    if (session.user) {
        return <>{children}</>
    } else if (session.signingIn) {
        return <p>Logging you in...</p>
    } else {
        return <Navigate to="/"></Navigate>
    }
}