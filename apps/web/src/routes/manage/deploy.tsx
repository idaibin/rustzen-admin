import {
    CloudUploadOutlined,
    DeleteOutlined,
    ExclamationCircleOutlined,
    UploadOutlined,
} from "@ant-design/icons";
import {
    ModalForm,
    ProFormText,
    ProFormTextArea,
    ProTable,
    type ActionType,
    type ProColumns,
} from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import {
    Button,
    Form,
    Segmented,
    Space,
    Tag,
    Upload,
    type UploadFile,
} from "antd";
import type { ReactElement } from "react";
import { useRef, useState } from "react";

import { appMessage, manageAPI } from "@/api";
import { AuthPopconfirm, AuthWrap } from "@/components/base-auth";
import {
    TABLE_ACTION_SPACE_SIZE,
    TableActionButton,
} from "@/components/base-button";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/manage/deploy")({
    component: DeployPage,
});

const componentOptions: Array<{ label: string; value: Deploy.Component }> = [
    { label: "Server", value: "server" },
    { label: "Web", value: "web" },
];

type ComponentFilter = "all" | Deploy.Component;

const componentFilterOptions: Array<{ label: string; value: ComponentFilter }> = [
    { label: "All", value: "all" },
    { label: "Server", value: "server" },
    { label: "Web", value: "web" },
];

const archOptions = [
    { label: "x86_64", value: "x86_64" },
    { label: "aarch64", value: "aarch64" },
];

function DeployPage() {
    const actionRef = useRef<ActionType>(null);
    const [component, setComponent] = useState<ComponentFilter>("all");
    const selectedComponent = component === "all" ? undefined : component;

    return (
        <ProTable<Deploy.Item>
            rowKey="id"
            search={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="Deploy Versions"
            columns={deployColumns}
            params={{ component: selectedComponent }}
            request={manageAPI.deploy.list}
            actionRef={actionRef}
            toolBarRender={() => [
                <Segmented
                    key="component"
                    value={component}
                    options={componentFilterOptions}
                    onChange={(value) => setComponent(value as ComponentFilter)}
                />,
                <AuthWrap key="upload" code="manage:deploy:create">
                    <UploadVersionModal
                        onSuccess={() => {
                            void actionRef.current?.reload();
                        }}
                    >
                        <Button type="primary" icon={<UploadOutlined />}>Upload Version</Button>
                    </UploadVersionModal>
                </AuthWrap>,
                <AuthWrap key="cleanup" code="manage:deploy:delete">
                    <CleanupButton
                        component={selectedComponent}
                        onSuccess={() => {
                            void actionRef.current?.reload();
                        }}
                    />
                </AuthWrap>,
            ]}
        />
    );
}

const deployColumns: ProColumns<Deploy.Item>[] = [
    {
        title: "Component",
        dataIndex: "component",
        width: 100,
        render: (_, record) => componentLabel(record.component),
    },
    {
        title: "Version",
        dataIndex: "version",
        width: 120,
        ellipsis: true,
    },
    {
        title: "Arch",
        dataIndex: "arch",
        width: 100,
    },
    {
        title: "Size",
        dataIndex: "fileSize",
        width: 100,
        render: (_, record) => formatFileSize(record.fileSize),
    },
    {
        title: "File Hash",
        dataIndex: "fileHash",
        width: 220,
        ellipsis: true,
        hideInTable: true,
    },
    {
        title: "Status",
        key: "status",
        width: 120,
        render: (_, record) => renderStatus(record),
    },
    {
        title: "Deployed By",
        dataIndex: "deployedBy",
        width: 120,
        render: (_, record) => record.deployedBy || "-",
    },
    {
        title: "Deployed At",
        dataIndex: "deployedAt",
        width: 180,
        render: (_, record) => formatDateTime(record.deployedAt),
    },
    {
        title: "Expired At",
        dataIndex: "expiredAt",
        width: 180,
        render: (_, record) => formatDateTime(record.expiredAt),
    },
    {
        title: "Notes",
        dataIndex: "notes",
        ellipsis: true,
        render: (_, record) => record.notes || "-",
    },
    {
        title: "Actions",
        key: "actions",
        width: 172,
        fixed: "right",
        render: (_dom, record, _index, action) => (
            <Space size={TABLE_ACTION_SPACE_SIZE}>
                <AuthPopconfirm
                    code="manage:deploy:run"
                    title="Deploy this version?"
                    description={
                        record.component === "web"
                            ? "Web deployment replaces the runtime web/dist directory."
                            : "Server deployment switches the runtime binary and restarts the service."
                    }
                    onConfirm={async () => {
                        const username = useAuthStore.getState().userInfo?.username || "developer";
                        await manageAPI.deploy.deploy(record.id, {
                            versionId: record.id,
                            deployedBy: username,
                        });
                        appMessage.success("Deploy task submitted");
                        void action?.reload();
                    }}
                >
                    <TableActionButton
                        color="blue"
                        label="Deploy"
                        icon={<CloudUploadOutlined />}
                        disabled={record.isExpired}
                    />
                </AuthPopconfirm>
                <AuthWrap code="manage:deploy:update">
                    <ExpireVersionModal
                        version={record}
                        onSuccess={() => {
                            void action?.reload();
                        }}
                    >
                        <TableActionButton
                            color="default"
                            label="Expire"
                            icon={<ExclamationCircleOutlined />}
                            disabled={record.isCurrent || record.isExpired}
                        />
                    </ExpireVersionModal>
                </AuthWrap>
                <AuthPopconfirm
                    code="manage:deploy:delete"
                    title="Delete this version?"
                    description="The saved deploy file will also be removed."
                    onConfirm={async () => {
                        await manageAPI.deploy.remove(record.id);
                        void action?.reload();
                    }}
                >
                    <TableActionButton
                        color="danger"
                        label="Delete"
                        icon={<DeleteOutlined />}
                        disabled={record.isCurrent}
                    />
                </AuthPopconfirm>
            </Space>
        ),
    },
];

interface UploadVersionFormValues {
    component: Deploy.Component;
    version: string;
    arch?: string;
    notes?: string;
    file?: UploadFile[];
}

function UploadVersionModal({
    children,
    onSuccess,
}: {
    children: ReactElement;
    onSuccess?: () => void;
}) {
    return (
        <ModalForm<UploadVersionFormValues>
            width={620}
            layout="horizontal"
            title="Upload Deploy Version"
            trigger={children}
            initialValues={{ arch: "x86_64" }}
            labelCol={{ span: 5 }}
            wrapperCol={{ span: 19 }}
            modalProps={{
                destroyOnHidden: true,
                forceRender: true,
                mask: { closable: false },
                okText: "Upload",
                cancelText: "Cancel",
            }}
            onFinish={async (values) => {
                const uploadFile = values.file?.[0]?.originFileObj;
                if (!uploadFile) {
                    appMessage.error("Please choose a deploy file");
                    return false;
                }

                await manageAPI.deploy.upload({
                    component: values.component,
                    version: values.version,
                    arch: values.component === "server" ? values.arch : undefined,
                    notes: values.notes,
                    file: uploadFile,
                });
                appMessage.success("Upload succeeded");
                onSuccess?.();
                return true;
            }}
        >
            <Form.Item
                name="component"
                label="Component"
                rules={[{ required: true, message: "Please select component" }]}
            >
                <Segmented options={componentOptions} />
            </Form.Item>
            <ProFormText
                name="version"
                label="Version"
                placeholder="v0.4.0"
                rules={[{ required: true, message: "Please enter version" }]}
            />
            <Form.Item
                name="arch"
                label="Arch"
                rules={[{ required: true, message: "Please select arch" }]}
            >
                <Segmented options={archOptions} />
            </Form.Item>
            <Form.Item
                name="file"
                label="File"
                valuePropName="fileList"
                getValueFromEvent={getUploadFileList}
                rules={[{ required: true, message: "Please choose deploy file" }]}
            >
                <Upload beforeUpload={() => false} maxCount={1}>
                    <Button icon={<UploadOutlined />}>Choose File</Button>
                </Upload>
            </Form.Item>
            <ProFormTextArea name="notes" label="Notes" placeholder="Optional notes" />
        </ModalForm>
    );
}

function ExpireVersionModal({
    version,
    children,
    onSuccess,
}: {
    version: Deploy.Item;
    children: ReactElement;
    onSuccess?: () => void;
}) {
    return (
        <ModalForm<Deploy.ExpireRequest>
            width={520}
            layout="horizontal"
            title={`Expire ${componentLabel(version.component)} ${version.version}`}
            trigger={children}
            disabled={version.isCurrent || version.isExpired}
            labelCol={{ span: 6 }}
            wrapperCol={{ span: 18 }}
            modalProps={{
                destroyOnHidden: true,
                mask: { closable: false },
                okText: "Expire",
                cancelText: "Cancel",
            }}
            onFinish={async (values) => {
                await manageAPI.deploy.expire(version.id, {
                    notes: values.notes || null,
                });
                onSuccess?.();
                return true;
            }}
        >
            <ProFormTextArea name="notes" label="Notes" placeholder="Optional reason" />
        </ModalForm>
    );
}

function CleanupButton({
    component,
    onSuccess,
}: {
    component?: Deploy.Component;
    onSuccess?: () => void;
}) {
    return (
        <AuthPopconfirm
            code="manage:deploy:delete"
            title="Clean expired versions?"
            description="Expired non-current version files will be removed."
            onConfirm={async () => {
                const count = await manageAPI.deploy.cleanup(component);
                appMessage.success(`Cleaned ${count} expired versions`);
                onSuccess?.();
            }}
        >
            <Button danger>Clean Expired</Button>
        </AuthPopconfirm>
    );
}

function renderStatus(record: Deploy.Item) {
    if (record.isCurrent) {
        return <Tag color="green">Current</Tag>;
    }
    if (record.isExpired) {
        return <Tag color="orange">Expired</Tag>;
    }
    if (record.isDeployed) {
        return <Tag color="blue">Deployed</Tag>;
    }
    return <Tag color="default">Uploaded</Tag>;
}

function componentLabel(component: Deploy.Component) {
    return component === "server" ? "Server" : "Web";
}

function formatFileSize(value: number) {
    if (value < 1024 * 1024) {
        return `${(value / 1024).toFixed(1)} KB`;
    }
    return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function formatDateTime(value?: string | null) {
    if (!value) {
        return "-";
    }
    return new Date(value).toLocaleString();
}

function getUploadFileList(event: UploadFile[] | { fileList?: UploadFile[] }) {
    if (Array.isArray(event)) {
        return event;
    }
    return event?.fileList;
}
