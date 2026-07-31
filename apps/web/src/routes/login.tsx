import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
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
        <main className="min-h-[100svh] bg-background text-foreground">
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
                                className="h-full w-full object-contain object-left"
                            />
                        </div>
                        <div className="-mt-9 flex w-full items-center gap-6 pl-3 xl:-mt-14 xl:gap-10">
                            <div className="shrink-0 text-[34px] leading-none whitespace-nowrap font-extrabold text-foreground xl:text-[40px]">
                                {t("让运维，更从容", "Operations, with confidence")}
                            </div>
                            <div className="h-12 w-px shrink-0 bg-border" />
                            <div className="grid min-w-0 gap-2">
                                <p className="m-0 text-[15px] leading-none font-semibold text-foreground xl:text-[16px]">
                                    {t("高效 · 可靠 · 智能", "Efficient · Reliable · Smart")}
                                </p>
                                <p className="m-0 text-[14px] leading-none text-muted-foreground xl:text-[15px]">
                                    {t(
                                        "统一运维管理平台，让管理更简单、更高效。",
                                        "A unified operations platform for simpler, more efficient management.",
                                    )}
                                </p>
                            </div>
                        </div>
                    </section>

                    <Card
                        className="w-full max-w-105 justify-self-center rounded-[18px] border border-border bg-card shadow-sm xl:max-w-133"
                        styles={{ body: { padding: 0 } }}
                        aria-label={t("登录", "Sign in")}
                    >
                        <div className="px-7 py-10 sm:px-12 sm:py-14 xl:px-16 xl:py-22">
                            <div className="mb-10 text-center xl:mb-12">
                                <Typography.Title
                                    level={1}
                                    className="m-0 text-[36px] font-extrabold leading-none text-foreground xl:text-[40px]"
                                >
                                    RustZen <span className="text-primary">Admin</span>
                                </Typography.Title>
                                <Typography.Text className="mt-6 block text-base leading-none text-muted-foreground">
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
                                            className="h-15 rounded-[10px] border-input text-base shadow-none hover:border-ring focus-visible:border-ring"
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
                                            className="h-15 rounded-[10px] border-input text-base shadow-none hover:border-ring focus-visible:border-ring"
                                        />
                                    </Form.Item>
                                </div>

                                <Form.Item className="!mb-0">
                                    <Button
                                        type="primary"
                                        htmlType="submit"
                                        block
                                        loading={isSubmitting}
                                        className="h-15 rounded-[10px] text-lg font-semibold shadow-sm"
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
