import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryKeys, reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { getSchedulePermissionState } from "./-schedule-permissions";
import { SchedulePanel } from "./-templates/schedule-panel";
import { TemplatesContent } from "./-templates/templates-content";

export const Route = createFileRoute("/reports/templates")({ component: FlowsPage });

function FlowsPage() {
    const client = useQueryClient();
    const checkPermissions = useAuthStore((state) => state.checkPermissions);
    const { canViewFlows, canViewSchedules } = getSchedulePermissionState(checkPermissions);
    const { data: systems = [] } = useQuery({
        ...reportsQueryOptions.systems(),
        enabled: canViewFlows,
    });
    const { data: flows = [], error, isPending, refetch } = useQuery({
        ...reportsQueryOptions.flows(),
        enabled: canViewFlows,
    });
    const refresh = () =>
        Promise.all([
            client.invalidateQueries({ queryKey: reportsQueryKeys.flows() }),
            client.invalidateQueries({ queryKey: reportsQueryKeys.flowOptions() }),
        ]);
    const refreshSystems = () => client.invalidateQueries({ queryKey: reportsQueryKeys.systems() });
    const clone = useMutation({
        mutationFn: (flow: Reports.Flow) =>
            reportsAPI.createFlow({
                systemId: flow.systemId,
                name: t(`${flow.name} 副本`, `${flow.name} copy`),
                steps: flow.steps,
            }),
        onSuccess: async () => {
            await refresh();
            appMessage.success(t("流程已复制", "Template copied"));
        },
    });

    if (!canViewFlows && !canViewSchedules) {
        return (
            <PageCard
                title={t("定时报表", "Scheduled reports")}
                description={t("查看现有报表模板的执行计划和最近结果。", "Inspect schedules and recent outcomes for existing report templates.")}
            >
                <DataState kind="permission" title={t("无报表查看权限", "Report view permission required")} />
            </PageCard>
        );
    }

    if (!canViewFlows) {
        return (
            <PageCard
                title={t("定时报表", "Scheduled reports")}
                description={t("查看现有报表模板的执行计划和最近结果。", "Inspect schedules and recent outcomes for existing report templates.")}
            >
                <AuthWrap
                    code="reports:schedule:view"
                    fallback={<DataState kind="permission" title={t("无计划查看权限", "Schedule view permission required")} />}
                >
                    <SchedulePanel />
                </AuthWrap>
            </PageCard>
        );
    }

    return (
        <TemplatesContent
            systems={systems}
            flows={flows}
            error={error}
            isPending={isPending}
            refetch={() => void refetch()}
            onClone={clone.mutate}
            onRefresh={refresh}
            onRefreshSystems={refreshSystems}
        />
    );
}
