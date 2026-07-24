import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute } from "@tanstack/react-router";
import { useNavigate } from "@tanstack/react-router";
import { Button, Card, Form, Input, Typography, type FormProps } from "antd";
import { useState } from "react";

import { authAPI } from "@/api";
import rustzenLogoUrl from "@/assets/rustzen-logo.png";
import { LanguageSwitch } from "@/components/language-switch";
import { ThemeSwitch } from "@/components/theme-provider";
import { APP_BRAND_NAME, RUSTZEN_BRAND_NAME } from "@/constant/brand";
import { t } from "@/lib/i18n";
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
    const [isSubmitting, setIsSubmitting] = useState(false);
    const { handleLogin } = useAuthStore();
    const currentYear = new Date().getFullYear();

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
        <main className="min-h-svh bg-background text-foreground">
            <div className="mx-auto flex min-h-svh w-full max-w-7xl flex-col px-5 py-5 sm:px-8">
                <header className="flex h-10 shrink-0 items-center gap-3">
                    <img
                        src={rustzenLogoUrl}
                        alt={RUSTZEN_BRAND_NAME}
                        className="size-10 object-contain"
                    />
                    <span className="text-[22px] font-bold leading-none">{APP_BRAND_NAME}</span>
                    <div className="ms-auto flex items-center gap-1">
                        <LanguageSwitch />
                        <ThemeSwitch />
                    </div>
                </header>

                <div className="flex flex-1 items-center justify-center py-10">
                    <Card
                        className="w-full max-w-100 rounded-lg border bg-card text-card-foreground shadow-sm sm:p-2"
                        aria-label={t("登录", "Sign in")}
                        styles={{ body: { padding: 0 } }}
                    >
                        <div className="grid gap-2 px-6 py-5">
                            <Typography.Title className="!mb-1" level={4}>
                                {t("登录", "Sign in")}
                            </Typography.Title>
                            <Typography.Text type="secondary">
                                {t(
                                    `使用你的 ${APP_BRAND_NAME} 账号继续。`,
                                    `Continue with your ${APP_BRAND_NAME} account.`,
                                )}
                            </Typography.Text>
                        </div>

                        <Form
                            layout="vertical"
                            autoComplete="off"
                            onFinish={onLogin}
                            className="grid gap-5 px-6 pb-6"
                            requiredMark={false}
                        >
                            <Form.Item
                                name="username"
                                label={t("用户名", "Username")}
                                rules={[
                                    {
                                        required: true,
                                        transform: (value) =>
                                            typeof value === "string" ? value.trim() : value,
                                        message: t("请输入用户名", "Please enter your username"),
                                    },
                                    {
                                        transform: (value) =>
                                            typeof value === "string" ? value.trim() : value,
                                        min: 3,
                                        message: t(
                                            "用户名至少需要 3 个字符",
                                            "Username must be at least 3 characters",
                                        ),
                                    },
                                ]}
                            >
                                <Input
                                    prefix={<UserOutlined />}
                                    placeholder={t("请输入用户名", "Enter your username")}
                                    autoComplete="username"
                                />
                            </Form.Item>

                            <Form.Item
                                name="password"
                                label={t("密码", "Password")}
                                rules={[
                                    {
                                        required: true,
                                        message: t("请输入密码", "Please enter your password"),
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
                                    prefix={<LockOutlined />}
                                    placeholder={t("请输入密码", "Enter your password")}
                                    autoComplete="current-password"
                                />
                            </Form.Item>

                            <Form.Item className="!mb-0">
                                <Button
                                    type="primary"
                                    htmlType="submit"
                                    block
                                    loading={isSubmitting}
                                >
                                    {isSubmitting
                                        ? t("正在登录...", "Signing in...")
                                        : t("登录", "Sign in")}
                                </Button>
                            </Form.Item>
                        </Form>
                    </Card>
                </div>

                <footer className="shrink-0 pb-3 text-sm text-muted-foreground">
                    © {currentYear} {RUSTZEN_BRAND_NAME}.{" "}
                    {t("保留所有权利。", "All rights reserved.")}
                </footer>
            </div>
        </main>
    );
}
