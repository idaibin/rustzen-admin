const TASK_REFRESH_INTERVAL = 1_000;

export const taskQueryKeys = {
    list: () => ["manage", "task"] as const,
    runs: (taskKey: string) => ["manage", "task", taskKey, "runs"] as const,
    runsPage: (taskKey: string, currentPage: number) =>
        [...taskQueryKeys.runs(taskKey), currentPage] as const,
};

export function taskListRefreshInterval(tasks: Pick<Task.Item, "running">[] | undefined) {
    return tasks?.some((task) => task.running) ? TASK_REFRESH_INTERVAL : false;
}

export function taskRunsRefreshInterval(
    open: boolean,
    runs: Pick<Task.RunItem, "status">[] | undefined,
) {
    return open && runs?.some((run) => run.status === "running") ? TASK_REFRESH_INTERVAL : false;
}
