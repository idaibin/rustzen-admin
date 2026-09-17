import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Button, Card, Form, Input, Typography, type FormProps } from "antd";
import { useState } from "react";

import { authAPI } from "@/api";
import loginIllustrationUrl from "@/assets/login-illustration.png";
import rustzenLogoUrl from "@/assets/rustzen-logo.png";
import { LanguageSwitch } from "@/components/language-switch";
import { ThemeSwitch } from "@/components/theme-provider";
import { APP_BRAND_NAME, RUSTZEN_BRAND_NAME } from "@/constant/brand";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/login")({
    component: LoginPage,
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
        <main className="min-h-[100svh] overflow-hidden bg-background text-foreground">
            <div className="relative mx-auto min-h-[100svh] w-full max-w-[1760px] px-6 sm:px-10 lg:px-12 xl:px-20">
                <header className="absolute inset-x-6 top-6 z-10 flex h-10 items-center gap-3 sm:inset-x-10 lg:inset-x-12 xl:inset-x-20 xl:top-8">
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

                <div className="mx-auto grid min-h-[100svh] w-full items-center gap-8 py-24 lg:grid-cols-[minmax(0,1fr)_420px] lg:gap-10 xl:grid-cols-[minmax(0,1fr)_500px] xl:gap-14 2xl:grid-cols-[minmax(0,1fr)_532px] 2xl:gap-20">
                    <section className="hidden min-w-0 self-stretch lg:flex lg:flex-col lg:justify-center">
                        <div className="h-105 xl:h-125 2xl:h-153">
                            <img
                                src={loginIllustrationUrl}
                                alt={`${APP_BRAND_NAME} Operations Management Platform`}
                                className="h-full w-full object-contain object-left drop-shadow-[0_28px_60px_rgba(31,95,191,0.12)]"
                            />
                        </div>
                        <div className="-mt-7 flex w-full items-center gap-5 pl-2 xl:-mt-9 xl:gap-6 2xl:-mt-14 2xl:gap-10 2xl:pl-3">
                            <div className="shrink-0 text-[28px] leading-none font-extrabold whitespace-nowrap text-foreground xl:text-[34px] 2xl:text-[40px]">
                                {t("让运维，更从容", "Operations, with confidence")}
                            </div>
                            <div className="h-11 w-px shrink-0 bg-border 2xl:h-12" />
                            <div className="grid min-w-0 gap-1.5 2xl:gap-2">
                                <p className="m-0 text-sm leading-none font-semibold text-foreground xl:text-[15px] 2xl:text-[16px]">
                                    {t("高效 · 可靠 · 智能", "Efficient · Reliable · Smart")}
                                </p>
                                <p className="m-0 text-xs leading-5 text-muted-foreground xl:text-[14px] 2xl:text-[15px]">
                                    {t(
                                        "统一运维管理平台，让管理更简单、更高效。",
                                        "A unified operations platform for simpler, more efficient management.",
                                    )}
                                </p>
                            </div>
                        </div>
                    </section>

                    <Card
                        className="w-full max-w-105 justify-self-center rounded-[18px] border border-border bg-card shadow-[0_24px_64px_rgba(31,65,116,0.1)] xl:max-w-125 2xl:max-w-133"
                        styles={{ body: { padding: 0 } }}
                        aria-label={t("登录", "Sign in")}
                    >
                        <div className="px-7 py-10 sm:px-10 sm:py-12 xl:px-14 xl:py-16 2xl:px-16 2xl:py-20">
                            <div className="mb-9 text-center xl:mb-10 2xl:mb-12">
                                <Typography.Title
                                    level={1}
                                    className="!m-0 !text-[34px] !leading-none !font-extrabold !text-foreground xl:!text-[38px] 2xl:!text-[40px]"
                                >
                                    {RUSTZEN_BRAND_NAME} <span className="text-primary">Admin</span>
                                </Typography.Title>
                                <Typography.Text className="mt-5 block text-base leading-none text-muted-foreground 2xl:mt-6">
                                    {t("欢迎来到", "Welcome to")} {APP_BRAND_NAME}
                                </Typography.Text>
                            </div>

                            <Form
                                layout="vertical"
                                onFinish={onLogin}
                                className="grid gap-6 2xl:gap-7"
                                requiredMark={false}
                            >
                                <div className="grid gap-3">
                                    <label
                                        className="text-base font-semibold text-foreground"
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
                                            prefix={
                                                <UserOutlined className="text-muted-foreground" />
                                            }
                                            placeholder={t("请输入用户名", "Enter your username")}
                                            autoComplete="username"
                                            className="h-14 rounded-[10px] border-input text-base shadow-none hover:border-ring focus-visible:border-ring 2xl:h-15"
                                        />
                                    </Form.Item>
                                </div>

                                <div className="grid gap-3">
                                    <div className="text-base leading-none">
                                        <label
                                            htmlFor="login_password"
                                            className="font-semibold text-foreground"
                                        >
                                            {t("密码", "Password")}
                                        </label>
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
                                            prefix={
                                                <LockOutlined className="text-muted-foreground" />
                                            }
                                            placeholder={t("请输入密码", "Enter your password")}
                                            autoComplete="current-password"
                                            className="h-14 rounded-[10px] border-input text-base shadow-none hover:border-ring focus-visible:border-ring 2xl:h-15"
                                        />
                                    </Form.Item>
                                </div>

                                <Form.Item className="!mb-0">
                                    <Button
                                        type="primary"
                                        htmlType="submit"
                                        block
                                        loading={isSubmitting}
                                        className="h-14 rounded-[10px] text-lg font-semibold shadow-[0_10px_22px_rgba(31,95,191,0.2)] 2xl:h-15"
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
