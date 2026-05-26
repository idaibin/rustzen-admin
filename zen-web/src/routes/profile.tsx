import { EditOutlined, LockOutlined } from "@ant-design/icons";
import { ModalForm, ProFormText } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Form, Tooltip } from "antd";

import { accountAPI, appMessage } from "@/api";
import { UserAvatar } from "@/components/base-user";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/profile")({
    component: ProfilePage,
});

function ProfilePage() {
    const { userInfo, updateUserInfo } = useAuthStore();
    const [passwordForm] = Form.useForm<Account.ChangePasswordRequest>();
    const [profileForm] = Form.useForm<Account.UpdateProfileRequest>();

    return (
        <div className="grid gap-5 p-6 xl:grid-cols-[minmax(520px,1fr)_420px]">
            <section className="rounded-lg bg-white p-6">
                <div className="mb-4 flex items-start justify-between gap-4">
                    <h1 className="text-xl font-semibold text-slate-900">User Profile</h1>
                    <div className="flex gap-2">
                        <ModalForm<Account.UpdateProfileRequest>
                            form={profileForm}
                            title="Edit Profile"
                            trigger={
                                <Tooltip title="Edit profile">
                                    <Button icon={<EditOutlined />} type="text" />
                                </Tooltip>
                            }
                            layout="horizontal"
                            labelCol={{ span: 5 }}
                            width={460}
                            initialValues={{
                                email: userInfo?.email,
                                realName: userInfo?.realName,
                            }}
                            modalProps={{ destroyOnHidden: true, centered: true }}
                            onOpenChange={(open) => {
                                if (!open) {
                                    profileForm.resetFields();
                                } else {
                                    profileForm.setFieldsValue({
                                        email: userInfo?.email,
                                        realName: userInfo?.realName,
                                    });
                                }
                            }}
                            onFinish={async (values) => {
                                const nextUserInfo = await accountAPI.updateProfile(values);
                                updateUserInfo(nextUserInfo);
                                appMessage.success("Profile updated");
                                return true;
                            }}
                        >
                            <ProFormText
                                name="email"
                                label="Email"
                                rules={[{ required: true, message: "Please enter email" }]}
                            />
                            <ProFormText name="realName" label="Real Name" />
                        </ModalForm>

                        <ModalForm<Account.ChangePasswordRequest>
                            form={passwordForm}
                            title="Change Password"
                            trigger={
                                <Tooltip title="Change password">
                                    <Button icon={<LockOutlined />} type="text" />
                                </Tooltip>
                            }
                            layout="vertical"
                            width={460}
                            modalProps={{ destroyOnHidden: true, centered: true }}
                            onOpenChange={(open) => {
                                if (!open) {
                                    passwordForm.resetFields();
                                }
                            }}
                            onFinish={async (values) => {
                                await accountAPI.changePassword(values);
                                passwordForm.resetFields();
                                appMessage.success("Password changed");
                                return true;
                            }}
                        >
                            <ProFormText.Password
                                name="currentPassword"
                                label="Current Password"
                                rules={[{ required: true, message: "Please enter current password" }]}
                            />
                            <ProFormText.Password
                                name="newPassword"
                                label="New Password"
                                rules={[{ required: true, message: "Please enter new password" }]}
                            />
                            <ProFormText.Password
                                name="confirmPassword"
                                label="Confirm Password"
                                rules={[
                                    { required: true, message: "Please confirm new password" },
                                    ({ getFieldValue }) => ({
                                        validator(_, value) {
                                            if (!value || getFieldValue("newPassword") === value) {
                                                return Promise.resolve();
                                            }
                                            return Promise.reject(
                                                new Error("The two passwords do not match"),
                                            );
                                        },
                                    }),
                                ]}
                            />
                        </ModalForm>
                    </div>
                </div>
                <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_220px]">
                    <div className="space-y-4">
                        <div>
                            <p className="mb-1 text-sm text-slate-500">Username</p>
                            <p className="text-sm text-slate-900">{userInfo?.username || "-"}</p>
                        </div>
                        <div>
                            <p className="mb-1 text-sm text-slate-500">Email</p>
                            <p className="text-sm text-slate-900">{userInfo?.email || "-"}</p>
                        </div>
                        <div>
                            <p className="mb-1 text-sm text-slate-500">Real Name</p>
                            <p className="text-sm text-slate-900">{userInfo?.realName || "-"}</p>
                        </div>
                    </div>
                    <div className="flex flex-col items-center">
                        <UserAvatar />
                    </div>
                </div>
            </section>
        </div>
    );
}
