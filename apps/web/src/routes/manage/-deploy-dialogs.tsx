import {
    CloudUploadOutlined,
    DeleteOutlined,
    FileDoneOutlined,
    FileSearchOutlined,
    UploadOutlined,
} from "@ant-design/icons";
import { Button, Form, Input, Modal, Upload } from "antd";
import type { UploadFile } from "antd/es/upload/interface";
import { useState } from "react";

import { appMessage, manageAPI } from "@/api";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DialogFooter } from "@/components/feedback/dialog-footer";
import { t } from "@/lib/i18n";

export function UploadVersionDialog({ onSuccess }: { onSuccess?: () => void }) {
    const [open, setOpen] = useState(false);
    const [version, setVersion] = useState("");
    const [arch, setArch] = useState("");
    const [notes, setNotes] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [fileList, setFileList] = useState<UploadFile[]>([]);

    const selectedFile = fileList[0]?.originFileObj ?? null;

    const reset = () => {
        setVersion("");
        setArch("");
        setNotes("");
        setFileList([]);
    };

    const submit = async () => {
        if (!version.trim()) {
            appMessage.error(t("请输入版本号", "Enter a version number"));
            return;
        }
        if (!selectedFile) {
            appMessage.error(t("请选择部署文件", "Select a deployment file"));
            return;
        }

        setSubmitting(true);
        try {
            await manageAPI.deploy.upload({
                version: version.trim(),
                arch: arch.trim() || undefined,
                notes: notes.trim() || undefined,
                file: selectedFile,
            });
            appMessage.success(t("上传成功", "Upload completed"));
            onSuccess?.();
            reset();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button type="default" icon={<UploadOutlined />} onClick={() => setOpen(true)}>
                {t("上传版本", "Upload version")}
            </Button>
            <Modal
                open={open}
                onCancel={() => {
                    setOpen(false);
                    reset();
                }}
                footer={null}
                title={t("上传完整发行包", "Upload complete release bundle")}
                width="650px"
                destroyOnHidden
            >
                <Form layout="vertical">
                    <Form.Item
                        label={t("版本", "Version")}
                        required
                        tooltip={t("用于展示与回滚核验", "Used for tracking and rollback checks")}
                    >
                        <Input
                            value={version}
                            placeholder="0.5.0"
                            onChange={(event) => setVersion(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("架构", "Architecture")}>
                        <Input
                            value={arch}
                            placeholder="x86_64"
                            onChange={(event) => setArch(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item
                        label={t("文件", "File")}
                        required
                        extra={t(
                            "上传一个包含 Admin、Monitor、Insights、Reports、Web，及部署文件的签名 tar 完整包。",
                            "Upload a signed complete tar bundle containing Admin, Monitor, Insights, Reports, Web, and deployment files.",
                        )}
                    >
                        <Upload.Dragger
                            multiple={false}
                            maxCount={1}
                            accept=".tar,application/x-tar"
                            beforeUpload={() => false}
                            fileList={fileList}
                            onChange={(info) => {
                                setFileList(info.fileList.slice(-1));
                            }}
                            onRemove={() => {
                                setFileList([]);
                            }}
                        >
                            <p className="ant-upload-drag-icon">
                                <FileSearchOutlined />
                            </p>
                            <p className="ant-upload-text">
                                {t("拖拽或点击选择文件", "Drag or click to select a file")}
                            </p>
                            <p className="ant-upload-hint">
                                {t("仅支持 .tar 文件", "Only .tar files are supported")}
                            </p>
                        </Upload.Dragger>
                    </Form.Item>
                    <Form.Item label={t("备注", "Notes")}>
                        <Input.TextArea
                            rows={3}
                            value={notes}
                            placeholder={t("可选备注", "Optional notes")}
                            onChange={(event) => setNotes(event.target.value)}
                        />
                    </Form.Item>
                    <DialogFooter
                        onCancel={() => {
                            setOpen(false);
                            reset();
                        }}
                        submitLabel={t("上传", "Upload")}
                        submitting={submitting}
                        onSubmit={submit}
                    />
                </Form>
            </Modal>
        </>
    );
}

export function DeployVersionDialog({
    record,
    onSuccess,
}: {
    record: Deploy.Item;
    onSuccess: () => void;
}) {
    const disabled = record.isExpired;
    const description = t(
        "rz 符号链接只切换一次，随后监控、分析、报表和管理服务依次通过健康检查门禁重启。门禁失败时会恢复原链接，并还原已进入重启流程的服务数据库。",
        "The rz symbolic link switches once, then Monitoring, Insights, Reports, and Admin restart in sequence behind health-check gates. If a gate fails, the original link and databases for services already in the restart flow are restored.",
    );

    const submit = async () => {
        await manageAPI.deploy.deploy(record.id);
        appMessage.success(t("部署任务已提交", "Deployment task submitted"));
        onSuccess();
    };

    return (
        <ConfirmDialog
            disabled={disabled}
            trigger={
                <Button
                    type="text"
                    icon={<CloudUploadOutlined />}
                    disabled={disabled}
                    aria-label={t("部署版本", "Deploy version")}
                />
            }
            title={t(
                `部署 ${componentLabel(record.component)} ${record.version}？`,
                `Deploy ${componentLabel(record.component)} ${record.version}?`,
            )}
            description={description}
            confirmLabel={t("部署", "Deploy")}
            onConfirm={submit}
        />
    );
}

export function ExpireVersionDialog({
    version,
    onSuccess,
}: {
    version: Deploy.Item;
    onSuccess?: () => void;
}) {
    const [open, setOpen] = useState(false);
    const [notes, setNotes] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const disabled = version.isCurrent || version.isExpired;

    const submit = async () => {
        setSubmitting(true);
        try {
            await manageAPI.deploy.expire(version.id, {
                notes: notes.trim() || null,
            });
            appMessage.success(t("版本已设为过期", "Version expired"));
            onSuccess?.();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                icon={<FileDoneOutlined />}
                disabled={disabled}
                onClick={() => setOpen(true)}
                aria-label={t("将版本设为过期", "Expire version")}
            />
            <Modal
                open={open}
                onCancel={() => {
                    setOpen(false);
                    setNotes("");
                }}
                footer={null}
                title={
                    <span>
                        {t(
                            `将 ${componentLabel(version.component)} ${version.version} 设为过期`,
                            `Expire ${componentLabel(version.component)} ${version.version}`,
                        )}
                    </span>
                }
            >
                <Form layout="vertical">
                    <Form.Item label={t("备注", "Notes")}>
                        <Input.TextArea
                            rows={3}
                            value={notes}
                            placeholder={t("可选原因", "Optional reason")}
                            onChange={(event) => setNotes(event.target.value)}
                        />
                    </Form.Item>
                    <DialogFooter
                        onCancel={() => {
                            setOpen(false);
                            setNotes("");
                        }}
                        submitLabel={t("设为过期", "Expire")}
                        submitting={submitting}
                        danger
                        onSubmit={submit}
                    />
                </Form>
            </Modal>
        </>
    );
}

export function DeleteVersionDialog({
    record,
    onSuccess,
}: {
    record: Deploy.Item;
    onSuccess: () => void;
}) {
    const disabled = record.isCurrent;

    const submit = async () => {
        await manageAPI.deploy.remove(record.id);
        appMessage.success(t("版本已删除", "Version deleted"));
        onSuccess();
    };

    return (
        <ConfirmDialog
            disabled={disabled}
            trigger={
                <Button
                    type="text"
                    icon={<DeleteOutlined />}
                    disabled={disabled}
                    danger
                    aria-label={t("删除版本", "Delete version")}
                />
            }
            title={t("删除版本", "Delete version")}
            description={t(
                `确定删除 ${componentLabel(record.component)} ${record.version}？系统会尽可能清理已保存的文件。`,
                `Delete ${componentLabel(record.component)} ${record.version}? The system will clean up saved files where possible.`,
            )}
            confirmLabel={t("删除", "Delete")}
            destructive
            onConfirm={submit}
        />
    );
}

export function CleanupDialog({
    component,
    onSuccess,
}: {
    component?: Deploy.Component;
    onSuccess?: () => void;
}) {
    const submit = async () => {
        const count = await manageAPI.deploy.cleanup(component);
        appMessage.success(t(`已清理 ${count} 个过期版本`, `Cleaned ${count} expired versions`));
        onSuccess?.();
    };

    return (
        <ConfirmDialog
            trigger={<Button type="default">{t("清理过期版本", "Clean expired versions")}</Button>}
            title={t("清理过期版本？", "Clean expired versions?")}
            description={t(
                "将从列表中移除非当前的过期版本，并尽可能清理已保存的文件。",
                "Remove non-current expired versions from the list and clean up saved files where possible.",
            )}
            confirmLabel={t("清理过期版本", "Clean expired versions")}
            destructive
            onConfirm={submit}
        />
    );
}

export function componentLabel(component: Deploy.Component) {
    return component === "release" ? t("发行包", "Release bundle") : component;
}
