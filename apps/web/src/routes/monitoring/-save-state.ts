export type FailedNetworkAction<T> = { type: "save"; values: T } | { type: "reset" };

export const failedNetworkAction = <T>(error: unknown, action: FailedNetworkAction<T>) =>
    error instanceof TypeError ? action : undefined;

export const retryFailedNetworkAction = <T>(
    canManage: boolean,
    action: FailedNetworkAction<T> | undefined,
    effects: { save: (values: T) => void; reset: () => void },
) => {
    if (!canManage || !action) return;
    if (action.type === "save") effects.save(action.values);
    else effects.reset();
};

export const shouldHydrateNodePolicy = (isNewNode: boolean, isTouched: boolean) =>
    isNewNode || !isTouched;

export const hasNodesBackgroundRefreshFailure = (data: unknown, error: unknown) =>
    data !== undefined && error != null;
