import { Alert, Snackbar } from "@mui/material";
import { createContext, ReactNode, useContext, useRef, useState } from "react";

interface Notification {
    class: "success" | "info" | "error",
    message: string,
    id: number,
    open: boolean,
}

export type CreateNotification = Omit<Notification, "id" | "open">;

export interface NotificationsContext {
    push: (notification: CreateNotification) => any,
}

const context = createContext<NotificationsContext>({
    push: () => { },
});

export function Notifications({ children }: { children: ReactNode }) {
    const [activeNotifications, setActiveNotifications] = useState<Notification[]>([])
    const lastId = useRef(0);

    function removeNotification(id: number) {
        setActiveNotifications((current) => {
            let cop = current.filter((v) => v.id !== id);
            return cop;
        })
    }
    function closeNotification(id: number) {
        setActiveNotifications((current) => {
            let cop = [...current];
            let copElemIndex = current.findIndex((v) => v.id === id);
            if (copElemIndex > -1) {
                cop[copElemIndex] = {
                    ...cop[copElemIndex],
                    open: false
                }

                setTimeout(() => removeNotification(id), 2000);
            }

            return cop;
        })
    }

    return <context.Provider value={{
        push: (notification) => {
            setActiveNotifications((current) => {
                const id = lastId.current++;
                return [...current, {
                    ...notification,
                    id: id,
                    open: true,
                }];
            })
        }
    }}>
        {children}
        {activeNotifications.map((v) => (
            <Snackbar key={v.id} open={v.open} autoHideDuration={6000} onClose={() => closeNotification(v.id)}>
                <Alert onClose={() => closeNotification(v.id)} severity={v.class} sx={{ width: '100%' }}>
                    {v.message}
                </Alert>
            </Snackbar>
        ))}
    </context.Provider>
}

export function UseNotifications() {
    return useContext(context);
}