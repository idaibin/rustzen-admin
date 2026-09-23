export const scheduleSaveErrorMessage = (error: unknown, fallbackError: string) =>
    error instanceof Error && error.message ? error.message : fallbackError;

export interface ScheduleDialogCycle {
    id: number;
    initialized: boolean;
}

export const initialScheduleDialogCycle: ScheduleDialogCycle = { id: 0, initialized: false };

export const beginScheduleDialogCycle = (cycle: ScheduleDialogCycle): ScheduleDialogCycle => ({
    id: cycle.id + 1,
    initialized: false,
});

export const endScheduleDialogCycle = beginScheduleDialogCycle;

export const isCurrentScheduleDialogCycle = (cycle: ScheduleDialogCycle, id: number) =>
    cycle.id === id;

export const shouldInitializeScheduleDraft = (cycle: ScheduleDialogCycle) => !cycle.initialized;

export const markScheduleDraftInitialized = (cycle: ScheduleDialogCycle): ScheduleDialogCycle => ({
    ...cycle,
    initialized: true,
});

interface ScheduleSaveEffects {
    refresh: () => Promise<unknown>;
    isCurrent: (cycle: number) => boolean;
    showSuccess: () => void;
    close: () => void;
    showError: (message: string) => void;
    fallbackError: string;
}

export const createScheduleSaveHandlers = (effects: ScheduleSaveEffects) => ({
    onSuccess: async (_data: unknown, variables: { cycle: number }) => {
        await effects.refresh();
        if (!effects.isCurrent(variables.cycle)) return;
        effects.showSuccess();
        effects.close();
    },
    onError: (error: unknown, variables: { cycle: number }) => {
        if (!effects.isCurrent(variables.cycle)) return;
        effects.showError(scheduleSaveErrorMessage(error, effects.fallbackError));
    },
});
