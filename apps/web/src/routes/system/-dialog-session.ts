export type DialogMode = "create" | "edit";

export interface DialogSessionTransition {
    sessionKey: string | null;
    shouldInitialize: boolean;
}

export function getDialogSessionTransition(
    open: boolean,
    mode: DialogMode,
    recordId: number | undefined,
    currentSessionKey: string | null,
): DialogSessionTransition {
    if (!open) {
        return { sessionKey: null, shouldInitialize: false };
    }

    const sessionKey = `${mode}:${recordId ?? "new"}`;
    return { sessionKey, shouldInitialize: currentSessionKey !== sessionKey };
}
