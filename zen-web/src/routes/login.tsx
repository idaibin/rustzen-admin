import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute } from "@tanstack/react-router";
import { useNavigate } from "@tanstack/react-router";
import { Button, Checkbox, Form, Input } from "antd";
import { useState } from "react";

import { authAPI } from "@/api";
import loginIllustrationUrl from "@/assets/login-illustration.png";
import rustzenLogoUrl from "@/assets/rustzen-logo.png";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/login")({
    component: () => <LoginPage />,
});

type LoginFormValues = Auth.LoginRequest & {
    remember?: boolean;
};

function LoginPage() {
    const navigate = useNavigate();
    const [isSubmitting, setIsSubmitting] = useState(false);
    const { handleLogin } = useAuthStore();
    const onLogin = async (values: LoginFormValues) => {
        setIsSubmitting(true);
        try {
            const res = await authAPI.login({
                username: values.username,
                password: values.password,
            });
            handleLogin(res.token, res.userInfo);
            void navigate({ to: "/", replace: true });
        } catch (error) {
            console.error("Login failed", error);
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <main className="relative h-screen overflow-hidden bg-[#f3f7ff] text-[#061634]">
            <div className="pointer-events-none absolute inset-y-0 right-0 w-[42vw] bg-[radial-gradient(#d6e3fb_1.3px,transparent_1.3px)] opacity-70 [background-size:30px_30px]" />
            <div className="pointer-events-none absolute inset-x-0 bottom-0 h-40 border-t border-[#dbe8ff]/60 bg-[repeating-radial-gradient(ellipse_at_12%_100%,rgba(45,112,255,0.12)_0,rgba(45,112,255,0.12)_1px,transparent_2px,transparent_24px)] opacity-60" />

            <div className="relative flex h-full flex-col px-6 py-6 sm:px-10 lg:px-14">
                <header className="flex shrink-0 items-center gap-3">
                    <span className="flex h-10 w-10 items-center justify-center overflow-hidden rounded-xl bg-white/70 shadow-[0_8px_24px_rgba(39,111,255,0.12)]">
                        <img
                            src={rustzenLogoUrl}
                            alt="Rustzen"
                            className="h-full w-full scale-[2.5] object-cover"
                        />
                    </span>
                    <span className="text-xl font-bold text-[#071836] sm:text-2xl">
                        Rustzen Admin
                    </span>
                </header>

                <div className="grid flex-1 items-center gap-8 py-6 lg:grid-cols-[minmax(0,1fr)_420px] lg:gap-8 lg:py-8 2xl:grid-cols-[minmax(0,1fr)_532px] 2xl:gap-14">
                    <section className="hidden min-w-0 lg:block" aria-label="产品视觉">
                        <div className="mx-auto max-w-[820px] rounded-[32px] border border-[#dbe8ff]/80 bg-white/45 p-6 backdrop-blur-sm 2xl:max-w-[900px]">
                            <img
                                src={loginIllustrationUrl}
                                alt="Rustzen Admin 运维管理平台"
                                className="mx-auto h-[420px] w-full object-contain drop-shadow-[0_34px_72px_rgba(38,103,255,0.16)] 2xl:h-[500px]"
                            />
                        </div>
                        <div className="mt-4 pl-8">
                            <div className="flex items-center gap-5 text-4xl font-extrabold text-[#061634]">
                                <span>高效</span>
                                <span className="text-2xl text-[#1577ff]">·</span>
                                <span>稳定</span>
                                <span className="text-2xl text-[#1577ff]">·</span>
                                <span>智能</span>
                            </div>
                            <p className="mt-4 text-xl text-[#71809c]">
                                一体化管理平台，让运维管理更简单
                            </p>
                        </div>
                    </section>

                    <section
                        className="mx-auto w-full max-w-[420px] rounded-[28px] bg-white px-6 py-8 shadow-[0_28px_80px_rgba(35,82,145,0.10)] sm:px-10 sm:py-14 2xl:max-w-[532px] 2xl:px-16"
                        aria-label="登录"
                    >
                        <div className="mb-9 text-center">
                            <h1 className="m-0 text-4xl font-extrabold text-[#061634]">
                                Rustzen <span className="text-[#1677ff]">Admin</span>
                            </h1>
                            <p className="mt-5 text-base text-[#8b98ae]">欢迎登录管理系统</p>
                        </div>

                        <Form<LoginFormValues>
                            name="login"
                            onFinish={onLogin}
                            autoComplete="off"
                            size="large"
                            layout="vertical"
                            initialValues={{ username: "superadmin", remember: true }}
                            requiredMark={false}
                        >
                            <Form.Item
                                name="username"
                                label={
                                    <span className="text-base font-semibold text-[#10213d]">
                                        用户名
                                    </span>
                                }
                                rules={[
                                    {
                                        required: true,
                                        message: "请输入用户名",
                                    },
                                    {
                                        min: 3,
                                        message: "用户名至少 3 个字符",
                                    },
                                ]}
                            >
                                <Input
                                    prefix={<UserOutlined className="text-[#8a9ab5]" />}
                                    placeholder="请输入用户名"
                                    autoComplete="username"
                                    className="h-[58px] rounded-xl border-[#dce4f1] px-4 text-base shadow-none hover:border-[#1677ff] focus:border-[#1677ff]"
                                />
                            </Form.Item>

                            <Form.Item
                                name="password"
                                label={
                                    <span className="text-base font-semibold text-[#10213d]">
                                        密码
                                    </span>
                                }
                                rules={[
                                    {
                                        required: true,
                                        message: "请输入密码",
                                    },
                                    {
                                        min: 6,
                                        message: "密码至少 6 个字符",
                                    },
                                ]}
                            >
                                <Input.Password
                                    prefix={<LockOutlined className="text-[#8a9ab5]" />}
                                    placeholder="请输入密码"
                                    autoComplete="current-password"
                                    className="h-[58px] rounded-xl border-[#dce4f1] px-4 text-base shadow-none hover:border-[#1677ff] focus:border-[#1677ff]"
                                />
                            </Form.Item>

                            <div className="mb-7 flex items-center justify-between">
                                <Form.Item name="remember" valuePropName="checked" noStyle>
                                    <Checkbox className="text-[#22324d]">记住我</Checkbox>
                                </Form.Item>
                                <span className="font-medium text-[#1677ff]">忘记密码?</span>
                            </div>

                            <Button
                                type="primary"
                                htmlType="submit"
                                loading={isSubmitting}
                                className="h-[58px] w-full rounded-xl bg-[#1677ff] text-lg font-semibold shadow-[0_12px_28px_rgba(22,119,255,0.25)]"
                            >
                                登 录
                            </Button>
                        </Form>
                    </section>
                </div>

                <footer className="shrink-0 text-sm text-[#7f8da7]">
                    © 2024 Rustzen. All rights reserved.
                </footer>
            </div>
        </main>
    );
}
