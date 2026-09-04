import { useQuery } from "@tanstack/react-query";
import { Button, Card, Typography } from "antd";
import { useEffect, useState } from "react";

import { reportsAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { t } from "@/lib/i18n";

import { isActiveRun } from "./status";

export function LiveFrame({ run }: { run?: Reports.Run }) {
    const { data, error, refetch } = useQuery({
        queryKey: ["reports", "live-frame", run?.id],
        queryFn: ({ signal }) => reportsAPI.liveFrame(run!.id, signal),
        enabled: Boolean(run),
        refetchInterval: isActiveRun(run?.status) ? 1000 : false,
    });
    const [source, setSource] = useState<string>();

    useEffect(() => setSource(undefined), [run?.id]);
    useEffect(() => {
        if (!data) return;
        const url = URL.createObjectURL(data);
        setSource(url);
        return () => URL.revokeObjectURL(url);
    }, [data]);

    return (
        <div>
            <Typography.Title level={5}>{t("实时画面", "Live view")}</Typography.Title>
            <Card className="h-80 overflow-auto">
                {source ? (
                    <img
                        src={source}
                        className="max-h-64 w-full object-contain"
                        alt={t("执行实时画面", "Live run view")}
                    />
                ) : error ? (
                    <DataState
                        kind="error"
                        title={t("实时画面加载失败", "Failed to load live view")}
                        action={
                            <Button type="primary" onClick={() => void refetch()}>
                                {t("重新加载", "Reload")}
                            </Button>
                        }
                        compact
                        className="h-full"
                    />
                ) : (
                    <DataState
                        kind={isActiveRun(run?.status) ? "processing" : "empty"}
                        title={
                            isActiveRun(run?.status)
                                ? t("正在等待浏览器画面", "Waiting for browser view")
                                : t("暂无实时画面", "No live view")
                        }
                        compact
                        className="h-full"
                    />
                )}
            </Card>
        </div>
    );
}
