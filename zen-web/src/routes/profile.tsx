import { ProForm, ProFormText } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Form } from "antd";

import { accountAPI, appMessage } from "@/api";
import { UserAvatar } from "@/components/base-user";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/profile")({
    component: ProfilePage,
});

function ProfilePage() {
    const { userInfo, updateUserInfo } = useAuthStore();
    const [profileForm] = Form.useForm<Account.UpdateProfileRequest>();
    const [passwordForm] = Form.useForm<Account.ChangePasswordRequest>();

    return (
        <div className="grid gap-5 p-6 xl:grid-cols-[minmax(520px,1fr)_420px]">
            <section className="rounded-lg bg-white p-6">
                <h1 className="mb-6 text-xl font-semibold text-slate-900">User Profile</h1>
                <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_220px]">
                    <ProForm<Account.UpdateProfileRequest>
                        form={profileForm}
                        layout="horizontal"
                        labelCol={{ span: 5 }}
                        initialValues={userInfo || undefined}
                        onFinish={async (values) => {
                            const nextUserInfo = await accountAPI.updateProfile(values);
                            updateUserInfo(nextUserInfo);
                            appMessage.success("Profile updated");
                            return true;
                        }}
                    >
                        <ProFormText name="username" label="Username" readonly />
                        <ProFormText
                            name="email"
                            label="Email"
                            rules={[{ required: true, message: "Please enter email" }]}
                        />
                        <ProFormText name="realName" label="Real Name" />
                    </ProForm>
                    <div className="flex flex-col items-center">
                        <UserAvatar />
                    </div>
                </div>
            </section>

            <section className="rounded-lg bg-white p-6">
                <h2 className="mb-6 text-lg font-semibold text-slate-900">Change Password</h2>
                <ProForm<Account.ChangePasswordRequest>
                    form={passwordForm}
                    layout="vertical"
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
                        rules={[{ required: true, message: "Please confirm new password" }]}
                    />
                </ProForm>
            </section>
        </div>
    );
}
