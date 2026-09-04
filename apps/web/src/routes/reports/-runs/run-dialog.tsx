import { PlayCircleOutlined } from "@ant-design/icons";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Modal, Select } from "antd";
import { useEffect, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { t } from "@/lib/i18n";

const defaultRunInput = JSON.stringify({ value: "" }, null, 2);

export function RunDialog({ flows }: { flows: Reports.Flow[] }) {
    const client = useQueryClient();
    const [open, setOpen] = useState(false);
    const [flowId, setFlowId] = useState("");
    const [inputJson, setInputJson] = useState(defaultRunInput);
    const mutation = useMutation({
        mutationFn: reportsAPI.createRun,
        onSuccess: async () => {
            await client.invalidateQueries({ queryKey: ["reports", "runs"] });
            appMessage.success(t("填报执行已进入队列", "Report run queued"));
            setOpen(false);
        },
    });

    const save = () => {
        try {
            const input = JSON.parse(inputJson) as Record<string, unknown>;
            mutation.mutate({ flowId, input });
        } catch {
            appMessage.error(t("输入内容必须是有效的 JSON", "Input must be valid JSON"));
        }
    };

    useEffect(() => {
        if (!open) return;
        setFlowId((previous) => previous || flows[0]?.id || "");
    }, [flows, open]);

    return (
        <>
            <Button
                type="primary"
                disabled={!flows.length}
                icon={<PlayCircleOutlined />}
                onClick={() => setOpen(true)}
            >
                {t("新建填报", "New report run")}
            </Button>
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                footer={null}
                title={t("开始填报", "Start report run")}
                width={760}
                destroyOnHidden
            >
                <p className="mb-4 text-sm text-muted-foreground">
                    {t(
                        "选择已校验的流程，并填写本次写入使用的输入数据。",
                        "Select a verified template and enter the input data for this run.",
                    )}
                </p>
                <Alert
                    className="mb-4"
                    type="warning"
                    showIcon
                    title={t(
                        "不要提交密码、Token、密钥或其他敏感信息。",
                        "Do not submit passwords, tokens, keys, or other sensitive information.",
                    )}
                />
                <Form layout="vertical">
                    <Form.Item label={t("流程", "Template")}>
                        <Select
                            value={flowId || undefined}
                            onChange={setFlowId}
                            options={flows.map((flow) => ({ value: flow.id, label: flow.name }))}
                            placeholder={t("选择模板", "Select template")}
                        />
                    </Form.Item>
                    <Form.Item label={t("输入 JSON", "Input JSON")}>
                        <Input.TextArea
                            className="font-mono"
                            rows={10}
                            value={inputJson}
                            onChange={(event) => setInputJson(event.target.value)}
                        />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button type="default" onClick={() => setOpen(false)}>
                            {t("取消", "Cancel")}
                        </Button>
                        <Button type="primary" loading={mutation.isPending} onClick={save}>
                            {t("提交执行", "Submit run")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}
