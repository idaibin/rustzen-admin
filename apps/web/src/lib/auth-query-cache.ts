import type { QueryClient } from "@tanstack/react-query";

interface AuthGenerationStore {
    getState: () => { authGeneration: number };
    subscribe: (listener: (state: { authGeneration: number }) => void) => () => void;
}

export const bindAuthenticatedQueryCache = (
    queryClient: QueryClient,
    store: AuthGenerationStore,
): (() => void) => {
    let generation = store.getState().authGeneration;
    return store.subscribe((state) => {
        if (state.authGeneration === generation) return;
        generation = state.authGeneration;
        queryClient.clear();
    });
};
