import { GlobalOutlined } from "@ant-design/icons";
import { useMutation } from "@tanstack/react-query";
import { Button, Form, Input, Modal } from "antd";
import { useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { t } from "@/lib/i18n";

export function TargetDialog({ onSaved }: { onSaved: () => Promise<unknown> }) {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");
    const [baseUrl, setBaseUrl] = useState("");
    const mutation = useMutation({
        mutationFn: () =>
            reportsAPI.createSystem({
                name: name.trim(),
                baseUrl: baseUrl.trim(),
                enabled: true,
            }),
        onSuccess: async () => {
            await onSaved();
            appMessage.success(t("目标系统已添加", "Target system added"));
            setOpen(false);
        },
    });

    return (
        <>
            <Button type="default" icon={<GlobalOutlined />} onClick={() => setOpen(true)}>
                {t("添加目标系统", "Add target system")}
            </Button>
            <Modal
                open={open}
                title={t("添加报表目标", "Add report target")}
                onCancel={() => {
                    setOpen(false);
                    setName("");
                    setBaseUrl("");
                }}
                footer={null}
                destroyOnHidden
            >
                <p className="mb-3 text-sm text-muted-foreground">
                    {t(
                        "模板只能在这个可信来源内导航。",
                        "Templates can only navigate within this trusted origin.",
                    )}
                </p>
                <Form layout="vertical">
                    <Form.Item label={t("名称", "Name")}>
                        <Input value={name} onChange={(event) => setName(event.target.value)} />
                    </Form.Item>
                    <Form.Item label={t("基础地址", "Base URL")}>
                        <Input
                            value={baseUrl}
                            placeholder="https://example.com"
                            onChange={(event) => setBaseUrl(event.target.value)}
                        />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button type="default" onClick={() => setOpen(false)}>
                            {t("取消", "Cancel")}
                        </Button>
                        <Button
                            type="primary"
                            loading={mutation.isPending}
                            disabled={!name.trim() || !baseUrl.trim()}
                            onClick={() => mutation.mutate()}
                        >
                            {t("添加目标系统", "Add target system")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}
