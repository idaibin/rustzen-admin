import { useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Typography } from "antd";
import { useState } from "react";

import { monitorAPI } from "@/api";
import { t } from "@/lib/i18n";

import {
    nodeIdPattern,
    nodeOnboardingCommand,
    normalizeControllerUrl,
} from "./-node-onboarding-command";

export function NodeOnboarding() {
    const [form] = Form.useForm<{ nodeId: string; controllerUrl: string }>();
    const [setup, setSetup] = useState<{ nodeId: string; command: string }>();
    const [checking, setChecking] = useState(false);
    const [result, setResult] = useState<"connected" | "waiting" | "error">();
    const client = useQueryClient();

    const check = async () => {
        if (!setup) return;
        setChecking(true);
        try {
            const nodes = await monitorAPI.nodes();
            client.setQueryData(["monitor", "nodes"], nodes);
            setResult(nodes.some((node) => node.nodeId === setup.nodeId) ? "connected" : "waiting");
        } catch {
            setResult("error");
        } finally {
            setChecking(false);
        }
    };

    return (
        <div className="space-y-5">
            <Typography.Paragraph type="secondary">
                {t(
                    "在目标主机启动 Agent，首次有效上报后自动添加到节点列表。",
                    "Start an agent on the target host. Its first accepted report adds the node to this list.",
                )}
            </Typography.Paragraph>
            <Form
                form={form}
                disabled={checking}
                layout="vertical"
                initialValues={{ controllerUrl: window.location.origin }}
                onValuesChange={() => {
                    setSetup(undefined);
                    setResult(undefined);
                }}
                onFinish={({ nodeId, controllerUrl }) => {
                    setSetup({ nodeId, command: nodeOnboardingCommand(nodeId, controllerUrl) });
                    setResult(undefined);
                }}
            >
                <Form.Item
                    name="nodeId"
                    label={t("节点标识", "Node ID")}
                    rules={[
                        {
                            required: true,
                            message: t("请输入唯一的节点标识", "Enter a unique node ID"),
                        },
                        {
                            pattern: nodeIdPattern,
                            message: t(
                                "限 1–128 位字母、数字、点、下划线或短横线",
                                "Use 1–128 letters, digits, dots, underscores or hyphens",
                            ),
                        },
                        {
                            validator: async (_, value) => {
                                const nodes = client.getQueryData<Monitor.Node[]>([
                                    "monitor",
                                    "nodes",
                                ]);
                                if (nodes?.some((node) => node.nodeId === value))
                                    throw new Error(
                                        t(
                                            "该节点标识已存在，请使用新的标识",
                                            "This node ID already exists; choose a new one",
                                        ),
                                    );
                            },
                        },
                    ]}
                >
                    <Input placeholder="node-01" autoComplete="off" maxLength={128} />
                </Form.Item>
                <Form.Item
                    name="controllerUrl"
                    label={t("控制台地址", "Console URL")}
                    extra={t(
                        "填写目标主机可访问的地址。远程主机不能使用此处的 localhost 或 127.0.0.1。",
                        "Use an address reachable from the target host. Remote hosts cannot use this console's localhost or 127.0.0.1 address.",
                    )}
                    rules={[
                        { required: true, message: t("请输入控制台地址", "Enter the console URL") },
                        {
                            validator: async (_, value) => {
                                try {
                                    normalizeControllerUrl(value ?? "");
                                } catch {
                                    throw new Error(
                                        t(
                                            "请输入有效的 HTTP(S) 地址，不含账号密码、查询参数或锚点",
                                            "Enter an HTTP(S) URL without credentials, query or fragment",
                                        ),
                                    );
                                }
                            },
                        },
                    ]}
                >
                    <Input placeholder="https://admin.example.com" autoComplete="off" />
                </Form.Item>
                <Button type="primary" htmlType="submit">
                    {t("生成接入命令", "Generate setup command")}
                </Button>
            </Form>
            {setup ? (
                <div className="space-y-4">
                    <Typography.Paragraph>
                        {t(
                            "在目标主机安装匹配系统与架构的 rz-monitor-agent，并加入 PATH。然后在 Bash 中执行以下命令，按提示输入与控制端一致的 Agent token。凭据仅在目标主机输入。",
                            "Install rz-monitor-agent for the target OS and architecture and add it to PATH. Run this command in Bash, then enter the controller's Agent token on that host.",
                        )}
                    </Typography.Paragraph>
                    <Typography.Paragraph
                        copyable={{ text: setup.command }}
                        className="rounded-lg border border-border p-4"
                    >
                        <pre className="overflow-x-auto whitespace-pre text-xs">
                            {setup.command}
                        </pre>
                    </Typography.Paragraph>
                    <Typography.Paragraph type="secondary">
                        {t(
                            "保持 Agent 运行，每 30 秒上报一次。检查接入会查询真实节点列表；生成命令不会创建空节点。",
                            "Keep the agent running; it reports every 30 seconds. Checking reads the actual node list; generating a command does not create an empty node.",
                        )}
                    </Typography.Paragraph>
                    <Button onClick={() => void check()} loading={checking}>
                        {t("检查接入", "Check connection")}
                    </Button>
                    {result ? (
                        <Alert
                            showIcon
                            type={
                                result === "connected"
                                    ? "success"
                                    : result === "error"
                                      ? "error"
                                      : "info"
                            }
                            title={
                                result === "connected"
                                    ? t("节点已接入，列表已更新", "Node connected; list updated")
                                    : result === "error"
                                      ? t(
                                            "接入检查失败，请重试",
                                            "Connection check failed; try again",
                                        )
                                      : t(
                                            "尚未收到该节点上报，请确认 Agent 正在运行及地址、token 正确",
                                            "No report received yet. Check that the agent is running and the address and token are correct",
                                        )
                            }
                        />
                    ) : null}
                </div>
            ) : null}
        </div>
    );
}
