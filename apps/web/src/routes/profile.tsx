import { EditOutlined, LockOutlined } from "@ant-design/icons";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Form, Input, Modal, Space, type FormProps } from "antd";
import { useState } from "react";

import { accountAPI, appMessage } from "@/api";
import { PageCard } from "@/components/page/page-card";
import { PageHeader } from "@/components/page/page-header";
import { UserAvatar } from "@/components/user";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/profile")({
    component: ProfilePage,
});

interface ProfileFormValues {
    email: string;
    realName?: string;
}

interface PasswordFormValues {
    currentPassword: string;
    newPassword: string;
    confirmPassword: string;
}

function ProfilePage() {
    const { userInfo, updateUserInfo } = useAuthStore();

    return (
        <div className="flex h-full min-h-0 flex-col gap-5 overflow-y-auto pr-1">
            <PageHeader
                title={t("个人资料", "Profile")}
                description={t(
                    "管理个人账号信息与登录凭据。",
                    "Manage your account information and sign-in credentials.",
                )}
            />

            <PageCard
                title={t("账号信息", "Account information")}
                description={t("查看并维护当前账号资料。", "View and update your account details.")}
                actions={
                    <Space size="small">
                        <EditProfileDialog userInfo={userInfo} onUpdated={updateUserInfo} />
                        <ChangePasswordDialog />
                    </Space>
                }
            >
                <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_220px]">
                    <div className="grid gap-4">
                        <ProfileField label={t("用户名", "Username")} value={userInfo?.username} />
                        <ProfileField label={t("邮箱", "Email")} value={userInfo?.email} />
                        <ProfileField
                            label={t("真实姓名", "Full name")}
                            value={userInfo?.realName}
                        />
                    </div>
                    <div className="flex flex-col items-center">
                        <UserAvatar />
                    </div>
                </div>
            </PageCard>
        </div>
    );
}

function EditProfileDialog({
    userInfo,
    onUpdated,
}: {
    userInfo: Auth.UserInfoResponse | null;
    onUpdated: (value: Auth.UserInfoResponse) => void;
}) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);
    const [form] = Form.useForm<ProfileFormValues>();

    const closeDialog = () => {
        form.resetFields();
        setOpen(false);
    };

    const openDialog = () => {
        form.setFieldsValue({
            email: userInfo?.email ?? "",
            realName: userInfo?.realName ?? "",
        });
        setOpen(true);
    };

    const submit: FormProps<ProfileFormValues>["onFinish"] = async (values) => {
        setSubmitting(true);
        try {
            const nextUserInfo = await accountAPI.updateProfile({
                email: values.email.trim(),
                realName: values.realName?.trim() || null,
            });
            onUpdated(nextUserInfo);
            appMessage.success(t("个人资料已更新", "Profile updated"));
            closeDialog();
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                icon={<EditOutlined />}
                onClick={openDialog}
                aria-label={t("编辑个人资料", "Edit profile")}
            >
                {t("编辑", "Edit")}
            </Button>
            <Modal
                open={open}
                onCancel={closeDialog}
                destroyOnHidden
                title={t("编辑个人资料", "Edit profile")}
                footer={null}
            >
                <Form
                    form={form}
                    onFinish={submit}
                    layout="vertical"
                    requiredMark={false}
                    initialValues={{
                        email: userInfo?.email ?? "",
                        realName: userInfo?.realName ?? "",
                    }}
                >
                    <Form.Item
                        name="email"
                        label={t("邮箱", "Email")}
                        rules={[
                            {
                                required: true,
                                message: t("请输入邮箱", "Enter an email address"),
                            },
                        ]}
                    >
                        <Input />
                    </Form.Item>
                    <Form.Item name="realName" label={t("真实姓名", "Full name")}>
                        <Input />
                    </Form.Item>
                    <Form.Item className="!mb-0">
                        <div className="flex items-center justify-end gap-2">
                            <Button onClick={closeDialog}>{t("取消", "Cancel")}</Button>
                            <Button type="primary" htmlType="submit" loading={submitting}>
                                {t("保存", "Save")}
                            </Button>
                        </div>
                    </Form.Item>
                </Form>
            </Modal>
        </>
    );
}

function ChangePasswordDialog() {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);
    const [form] = Form.useForm<PasswordFormValues>();

    const closeDialog = () => {
        form.resetFields();
        setOpen(false);
    };

    const submit: FormProps<PasswordFormValues>["onFinish"] = async (values) => {
        if (values.newPassword !== values.confirmPassword) {
            appMessage.error(t("两次输入的密码不一致", "The passwords do not match"));
            return;
        }

        setSubmitting(true);
        try {
            await accountAPI.changePassword({
                currentPassword: values.currentPassword,
                newPassword: values.newPassword,
                confirmPassword: values.confirmPassword,
            });
            appMessage.success(t("密码已修改", "Password changed"));
            closeDialog();
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                icon={<LockOutlined />}
                onClick={() => setOpen(true)}
                aria-label={t("修改密码", "Change password")}
            >
                {t("修改密码", "Change password")}
            </Button>
            <Modal
                open={open}
                onCancel={closeDialog}
                destroyOnHidden
                title={t("修改密码", "Change password")}
                footer={null}
            >
                <Form form={form} onFinish={submit} layout="vertical" requiredMark={false}>
                    <Form.Item
                        name="currentPassword"
                        label={t("当前密码", "Current password")}
                        rules={[
                            {
                                required: true,
                                message: t("请填写全部密码字段", "Complete all password fields"),
                            },
                        ]}
                    >
                        <Input.Password />
                    </Form.Item>
                    <Form.Item
                        name="newPassword"
                        label={t("新密码", "New password")}
                        rules={[
                            {
                                required: true,
                                message: t("请填写全部密码字段", "Complete all password fields"),
                            },
                        ]}
                    >
                        <Input.Password />
                    </Form.Item>
                    <Form.Item
                        name="confirmPassword"
                        label={t("确认密码", "Confirm password")}
                        rules={[
                            {
                                required: true,
                                message: t("请填写全部密码字段", "Complete all password fields"),
                            },
                        ]}
                    >
                        <Input.Password />
                    </Form.Item>
                    <Form.Item className="!mb-0">
                        <div className="flex items-center justify-end gap-2">
                            <Button onClick={closeDialog}>{t("取消", "Cancel")}</Button>
                            <Button type="primary" htmlType="submit" loading={submitting}>
                                {t("保存", "Save")}
                            </Button>
                        </div>
                    </Form.Item>
                </Form>
            </Modal>
        </>
    );
}

function ProfileField({ label, value }: { label: string; value?: string | null }) {
    return (
        <div>
            <p className="mb-1 text-sm text-muted-foreground">{label}</p>
            <p className="text-sm">{value || "-"}</p>
        </div>
    );
}
