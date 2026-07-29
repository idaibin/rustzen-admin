import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Button, Card, Flex, Form, Input, Space, Typography, type FormProps } from "antd";
import { useState } from "react";

import { authAPI } from "@/api";
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
        <main className="min-h-[100svh] p-4">
            <Flex className="min-h-[calc(100svh-2rem)]" align="center" justify="center">
                <Card
                    className="w-full max-w-105"
                    title={
                        <Space>
                            <img
                                src={rustzenLogoUrl}
                                alt={APP_BRAND_NAME}
                                className="size-8 object-contain"
                            />
                            <Typography.Text strong>{APP_BRAND_NAME}</Typography.Text>
                        </Space>
                    }
                    extra={
                        <Space size="small">
                            <LanguageSwitch />
                            <ThemeSwitch />
                        </Space>
                    }
                    aria-label={t("登录", "Sign in")}
                >
                    <Typography.Title level={2}>{t("登录", "Sign in")}</Typography.Title>
                    <Typography.Paragraph type="secondary">
                        {t("登录统一运维管理平台", "Sign in to the operations management platform")}
                    </Typography.Paragraph>

                    <Form<LoginPayload>
                        layout="vertical"
                        autoComplete="off"
                        onFinish={onLogin}
                        requiredMark={false}
                    >
                        <Form.Item
                            label={t("用户名", "Username")}
                            name="username"
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
                                size="large"
                            />
                        </Form.Item>

                        <Form.Item
                            label={t("密码", "Password")}
                            name="password"
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
                                size="large"
                            />
                        </Form.Item>

                        <Form.Item>
                            <Button
                                type="primary"
                                htmlType="submit"
                                block
                                size="large"
                                loading={isSubmitting}
                            >
                                {isSubmitting
                                    ? t("正在登录...", "Signing in...")
                                    : t("登录", "Sign in")}
                            </Button>
                        </Form.Item>
                    </Form>
                </Card>
            </Flex>
        </main>
    );
}
