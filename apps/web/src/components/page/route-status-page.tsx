import { useNavigate, useRouter } from "@tanstack/react-router";
import { Button, Result } from "antd";

import { t } from "@/lib/i18n";

interface RouteStatusPageProps {
    code: string;
    title: string;
    description: string;
}

export function RouteStatusPage({ code, title, description }: RouteStatusPageProps) {
    const navigate = useNavigate();
    const { history } = useRouter();
    const isPermissionCode = code === "403";

    return (
        <section className="flex h-full min-h-0 flex-col items-center justify-center px-4">
            <Result
                status={isPermissionCode ? "403" : "error"}
                title={isPermissionCode ? `${code} · ${title}` : title}
                subTitle={description}
                extra={
                    <>
                        <Button type="default" onClick={() => history.go(-1)}>
                            {t("返回上一页", "Go back")}
                        </Button>
                        <Button type="primary" onClick={() => void navigate({ to: "/" })}>
                            {t("返回首页", "Back to home")}
                        </Button>
                    </>
                }
            />
        </section>
    );
}
