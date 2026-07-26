import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute } from "@tanstack/react-router";
import { useNavigate } from "@tanstack/react-router";
import { Button, Card, Form, Input, Typography, type FormProps } from "antd";
import { useState } from "react";

import { authAPI } from "@/api";
import loginIllustrationUrl from "@/assets/login-illustration.png";
import rustzenLogoUrl from "@/assets/rustzen-logo.png";
import { LanguageSwitch } from "@/components/language-switch";
import { ThemeSwitch } from "@/components/theme-provider";
import { APP_BRAND_NAME } from "@/constant/brand";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/login")({
    component: () => <LoginPage />,
});

interface LoginPayload {
    username: string;
    password: string;
}

function LoginPage() {
    const navigate = useNavigate();
    useLocale();
    const [isSubmitting, setIsSubmitting] = useState(false);
    const { handleLogin } = useAuthStore();

    const onLogin: FormProps<LoginPayload>["onFinish"] = async ({ username, password }) => {
        setIsSubmitting(true);
        try {
            const res = await authAPI.login({
                username: username.trim(),
                password,
            });
            handleLogin(res.token, res.userInfo);
            void navigate({ to: "/", replace: true });
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <main className="relative min-h-[100svh] overflow-hidden bg-[#f3f7ff] text-[#061634]">
            <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_28%_26%,rgba(255,255,255,0.92)_0,rgba(255,255,255,0.28)_24%,transparent_44%),linear-gradient(120deg,#f5f8ff_0%,#eef4ff_45%,#f8fbff_100%)]" />
            <div className="pointer-events-none absolute right-0 top-0 h-[58vh] w-[42vw] bg-[radial-gradient(#d9e6ff_1.4px,transparent_1.4px)] opacity-70 [background-size:31px_31px]" />
            <div className="pointer-events-none absolute bottom-[-16vh] left-[-8vw] h-[46vh] w-[78vw] rounded-[50%] border border-[#d9e6fb]/70" />
            <div className="pointer-events-none absolute bottom-[-19vh] left-[-5vw] h-[42vh] w-[72vw] rounded-[50%] border border-[#d9e6fb]/55" />
            <div className="pointer-events-none absolute bottom-[-23vh] left-[1vw] h-[38vh] w-[64vw] rounded-[50%] border border-[#d9e6fb]/40" />

            <div className="relative mx-auto min-h-[100svh] w-full max-w-[1760px] px-7 sm:px-12 lg:px-14 xl:px-20">
                <header className="absolute inset-x-7 top-8 z-10 flex h-10 items-center gap-3 sm:inset-x-12 lg:inset-x-14 xl:inset-x-20">
                    <img
                        src={rustzenLogoUrl}
                        alt={APP_BRAND_NAME}
                        className="size-10 object-contain"
                    />
                    <span className="text-[22px] font-bold leading-none">{APP_BRAND_NAME}</span>
                    <div className="ms-auto flex items-center gap-1">
                        <LanguageSwitch />
                        <ThemeSwitch />
                    </div>
                </header>

                <div className="mx-auto grid min-h-[100svh] w-full items-center gap-8 py-24 min-[1440px]:grid-cols-[minmax(0,1fr)_532px] min-[1440px]:gap-14 2xl:gap-20">
                    <section className="hidden min-w-0 self-stretch min-[1440px]:flex min-[1440px]:flex-col min-[1440px]:justify-center">
                        <div className="h-125 xl:h-153">
                            <img
                                src={loginIllustrationUrl}
                                alt={`${APP_BRAND_NAME} Operations Management Platform`}
                                className="h-full w-full object-contain object-left drop-shadow-[0_34px_72px_rgba(38,103,255,0.12)]"
                            />
                        </div>
                        <div className="-mt-9 flex w-full items-center gap-6 pl-3 xl:-mt-14 xl:gap-10">
                            <div className="shrink-0 text-[34px] leading-none whitespace-nowrap font-extrabold text-[#061634] xl:text-[40px]">
                                {t("让运维，更从容", "Operations, with confidence")}
                            </div>
                            <div className="h-12 w-px shrink-0 bg-[#bdd5ff]" />
                            <div className="grid min-w-0 gap-2">
                                <p className="m-0 text-[15px] leading-none font-semibold text-[#1b365f] xl:text-[16px]">
                                    {t("高效 · 可靠 · 智能", "Efficient · Reliable · Smart")}
                                </p>
                                <p className="m-0 text-[14px] leading-none text-[#637eaa] xl:text-[15px]">
                                    {t(
                                        "统一运维管理平台，让管理更简单、更高效。",
                                        "A unified operations platform for simpler, more efficient management.",
                                    )}
                                </p>
                            </div>
                        </div>
                    </section>

                    <Card
                        className="w-full max-w-105 justify-self-center rounded-[18px] border-0 bg-white shadow-[0_28px_76px_rgba(45,88,150,0.09)] xl:max-w-133"
                        styles={{ body: { padding: 0 } }}
                        aria-label={t("登录", "Sign in")}
                    >
                        <div className="px-7 py-10 sm:px-12 sm:py-14 xl:px-16 xl:py-22">
                            <div className="mb-10 text-center xl:mb-12">
                                <Typography.Title
                                    level={1}
                                    className="m-0 text-[36px] font-extrabold leading-none text-[#061634] xl:text-[40px]"
                                >
                                    RustZen <span className="text-[#1677ff]">Admin</span>
                                </Typography.Title>
                                <Typography.Text className="mt-6 block text-base leading-none text-[#8b98ae]">
                                    {t("欢迎来到", "Welcome to")} {APP_BRAND_NAME}
                                </Typography.Text>
                            </div>

                            <Form
                                layout="vertical"
                                autoComplete="off"
                                onFinish={onLogin}
                                className="grid gap-7"
                                requiredMark={false}
                            >
                                <div className="grid gap-3">
                                    <label
                                        className="text-base font-semibold text-[#10213d]"
                                        htmlFor="login_username"
                                    >
                                        {t("用户名", "Username")}
                                    </label>
                                    <Form.Item
                                        name="username"
                                        className="!mb-0"
                                        rules={[
                                            {
                                                required: true,
                                                transform: (value) =>
                                                    typeof value === "string"
                                                        ? value.trim()
                                                        : value,
                                                message: t(
                                                    "请输入用户名",
                                                    "Please enter your username",
                                                ),
                                            },
                                            {
                                                transform: (value) =>
                                                    typeof value === "string"
                                                        ? value.trim()
                                                        : value,
                                                min: 3,
                                                message: t(
                                                    "用户名至少需要 3 个字符",
                                                    "Username must be at least 3 characters",
                                                ),
                                            },
                                        ]}
                                    >
                                        <Input
                                            id="login_username"
                                            prefix={<UserOutlined className="text-[#8a9ab5]" />}
                                            placeholder={t("请输入用户名", "Enter your username")}
                                            autoComplete="username"
                                            className="h-15 rounded-[10px] border-[#dce4f1] text-base shadow-none hover:border-[#1677ff] focus-visible:border-[#1677ff]"
                                        />
                                    </Form.Item>
                                </div>

                                <div className="grid gap-3">
                                    <div className="flex items-center justify-between text-base leading-none">
                                        <label
                                            htmlFor="login_password"
                                            className="font-semibold text-[#10213d]"
                                        >
                                            {t("密码", "Password")}
                                        </label>
                                        <span className="text-sm font-medium text-[#8b98ae]">
                                            {t("忘记密码？", "Forgot password?")}
                                        </span>
                                    </div>
                                    <Form.Item
                                        name="password"
                                        className="!mb-0"
                                        rules={[
                                            {
                                                required: true,
                                                message: t(
                                                    "请输入密码",
                                                    "Please enter your password",
                                                ),
                                            },
                                            {
                                                min: 6,
                                                message: t(
                                                    "密码至少需要 6 个字符",
                                                    "Password must be at least 6 characters",
                                                ),
                                            },
                                        ]}
                                    >
                                        <Input.Password
                                            id="login_password"
                                            prefix={<LockOutlined className="text-[#8a9ab5]" />}
                                            placeholder={t("请输入密码", "Enter your password")}
                                            autoComplete="current-password"
                                            className="h-15 rounded-[10px] border-[#dce4f1] text-base shadow-none hover:border-[#1677ff] focus-visible:border-[#1677ff]"
                                        />
                                    </Form.Item>
                                </div>

                                <Form.Item className="!mb-0">
                                    <Button
                                        type="primary"
                                        htmlType="submit"
                                        block
                                        loading={isSubmitting}
                                        className="h-15 rounded-[10px] bg-[#1677ff] text-lg font-semibold shadow-[0_12px_22px_rgba(22,119,255,0.24)] hover:bg-[#1677ff]/90"
                                    >
                                        {isSubmitting
                                            ? t("正在登录...", "Signing in...")
                                            : t("登录", "Sign in")}
                                    </Button>
                                </Form.Item>
                            </Form>
                        </div>
                    </Card>
                </div>
            </div>
        </main>
    );
}
