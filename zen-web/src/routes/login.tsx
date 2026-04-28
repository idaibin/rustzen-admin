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
        <main className="relative min-h-screen overflow-hidden bg-[#f3f7ff] text-[#061634]">
            <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_28%_26%,rgba(255,255,255,0.92)_0,rgba(255,255,255,0.28)_24%,transparent_44%),linear-gradient(120deg,#f5f8ff_0%,#eef4ff_45%,#f8fbff_100%)]" />
            <div className="pointer-events-none absolute right-0 top-0 h-[58vh] w-[42vw] bg-[radial-gradient(#d9e6ff_1.4px,transparent_1.4px)] opacity-70 [background-size:31px_31px]" />
            <div className="pointer-events-none absolute bottom-[-16vh] left-[-8vw] h-[46vh] w-[78vw] rounded-[50%] border border-[#d9e6fb]/70" />
            <div className="pointer-events-none absolute bottom-[-19vh] left-[-5vw] h-[42vh] w-[72vw] rounded-[50%] border border-[#d9e6fb]/55" />
            <div className="pointer-events-none absolute bottom-[-23vh] left-[1vw] h-[38vh] w-[64vw] rounded-[50%] border border-[#d9e6fb]/40" />

            <div className="relative flex min-h-screen flex-col px-7 py-8 sm:px-12 lg:px-14 xl:px-20">
                <header className="flex h-10 shrink-0 items-center gap-3">
                    <span className="flex h-10 w-10 items-center justify-center">
                        <img
                            src={rustzenLogoUrl}
                            alt="Rustzen"
                            className="h-10 w-10 object-contain"
                        />
                    </span>
                    <span className="text-[22px] font-bold leading-none text-[#071836]">
                        Rustzen Admin
                    </span>
                </header>

                <div className="grid flex-1 items-center gap-8 py-7 lg:grid-cols-[minmax(560px,1fr)_420px] lg:gap-14 xl:grid-cols-[minmax(680px,1fr)_532px] xl:gap-20">
                    <section className="hidden min-w-0 self-stretch lg:flex lg:flex-col lg:justify-center">
                        <div className="h-[500px] xl:h-[610px]">
                            <img
                                src={loginIllustrationUrl}
                                alt="Rustzen Admin 运维管理平台"
                                className="h-full w-full object-contain object-left drop-shadow-[0_34px_72px_rgba(38,103,255,0.12)]"
                            />
                        </div>
                        <div className="-mt-9 pl-3 xl:-mt-14">
                            <div className="flex items-center gap-5 text-[34px] font-extrabold leading-none text-[#061634] xl:text-[38px]">
                                <span>高效</span>
                                <span className="text-[28px] text-[#1677ff]">•</span>
                                <span>稳定</span>
                                <span className="text-[28px] text-[#1677ff]">•</span>
                                <span>智能</span>
                            </div>
                            <p className="mt-5 text-[22px] leading-none text-[#7484a0]">
                                一体化管理平台，让运维管理更简单
                            </p>
                        </div>
                    </section>

                    <section
                        className="mx-auto w-full max-w-[420px] rounded-[18px] bg-white px-7 py-10 shadow-[0_28px_76px_rgba(45,88,150,0.09)] sm:px-12 sm:py-14 xl:max-w-[532px] xl:px-[62px] xl:py-[86px]"
                        aria-label="登录"
                    >
                        <div className="mb-10 text-center xl:mb-12">
                            <h1 className="m-0 text-[36px] font-extrabold leading-none text-[#061634] xl:text-[40px]">
                                Rustzen <span className="text-[#1677ff]">Admin</span>
                            </h1>
                            <p className="mt-6 text-base leading-none text-[#8b98ae]">
                                欢迎登录管理系统
                            </p>
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
                                className="mb-8"
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
                                    className="h-[60px] rounded-[10px] border-[#dce4f1] px-4 text-base shadow-none hover:border-[#1677ff] focus:border-[#1677ff]"
                                />
                            </Form.Item>

                            <Form.Item
                                name="password"
                                className="mb-7"
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
                                    className="h-[60px] rounded-[10px] border-[#dce4f1] px-4 text-base shadow-none hover:border-[#1677ff] focus:border-[#1677ff]"
                                />
                            </Form.Item>

                            <div className="mb-8 flex items-center justify-between text-base leading-none">
                                <Form.Item name="remember" valuePropName="checked" noStyle>
                                    <Checkbox className="text-[#22324d]">记住我</Checkbox>
                                </Form.Item>
                                <span className="font-medium text-[#1677ff]">忘记密码?</span>
                            </div>

                            <Button
                                type="primary"
                                htmlType="submit"
                                loading={isSubmitting}
                                className="h-[60px] w-full rounded-[10px] bg-[#1677ff] text-lg font-semibold shadow-[0_12px_22px_rgba(22,119,255,0.24)]"
                            >
                                登 录
                            </Button>
                        </Form>
                    </section>
                </div>

                <footer className="shrink-0 pb-3 text-sm text-[#7f8da7]">
                    © 2024 Rustzen. All rights reserved.
                </footer>
            </div>
        </main>
    );
}
