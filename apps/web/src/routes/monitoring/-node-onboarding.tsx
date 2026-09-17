import { Alert, Button, Steps, Typography } from "antd";

import { t, useLocale } from "@/lib/i18n";

export const onboardingStepCopy = [
    [
        "获取已签名的 node-agent 归档及其 release manifest。",
        "Obtain the signed node-agent archive and its release manifest.",
    ],
    [
        "获取匹配的签名 envelope、受信公钥和 key ID。",
        "Obtain the matching signature envelope, trusted public key, and key ID.",
    ],
    [
        "运行 rz apply，创建固定服务身份，再运行 prepare-monitor-agent-access。",
        "Run rz apply, create the fixed service identity, and run prepare-monitor-agent-access.",
    ],
    [
        "使用 rz pin-monitor-controller 固定已签名的 Controller tuple。",
        "Pin the signed Controller tuple with rz pin-monitor-controller.",
    ],
    [
        "创建 root-only 配置源，包含 RUSTZEN_ENV、RUSTZEN_MONITOR_AGENT_TOKEN、RUSTZEN_MONITOR_CONTROLLER_URL 和 RUSTZEN_MONITOR_NODE_ID。",
        "Create a root-only configuration source with RUSTZEN_ENV, RUSTZEN_MONITOR_AGENT_TOKEN, RUSTZEN_MONITOR_CONTROLLER_URL, and RUSTZEN_MONITOR_NODE_ID.",
    ],
    [
        "运行 rz activate-monitor-agent，并等待 Controller 接受第一条报告。",
        "Run rz activate-monitor-agent and wait for the first accepted report.",
    ],
] as const;

export function NodeOnboarding({ onClose }: { onClose: () => void }) {
    useLocale();
    const onboardingSteps = onboardingStepCopy.map(([zh, en]) => t(zh, en));
    return (
        <div className="flex h-full flex-col gap-5">
            <div className="min-h-0 flex-1 space-y-5 overflow-y-auto">
                <Typography.Paragraph type="secondary">
                    {t(
                        "控制台不能安全获取已签名离线文件，也不能在目标主机创建 root-only 密钥文件。请由发布操作员在目标主机完成以下步骤。",
                        "The console cannot securely obtain offline signed files or create a root-only secret file on the target host. Have the release operator complete these steps on that host.",
                    )}
                </Typography.Paragraph>
                <Steps
                    orientation="vertical"
                    size="small"
                    items={onboardingSteps.map((content) => ({ content }))}
                />
                <Alert
                    showIcon
                    type="info"
                    title={t("执行入口尚不可用", "Execution is not available yet")}
                    description={t(
                        "缺少已签名文件输入和本地 root-only 配置边界。页面不会显示 token，也不会生成或复制启动命令。",
                        "Signed file inputs and a local root-only configuration boundary are unavailable. This page shows no token and generates no executable command.",
                    )}
                />
            </div>
            <div className="flex shrink-0 justify-end border-t border-border pt-4">
                <Button onClick={onClose}>{t("关闭", "Close")}</Button>
            </div>
        </div>
    );
}
