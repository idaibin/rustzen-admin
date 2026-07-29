import {
    AppstoreOutlined,
    LockOutlined,
    SafetyCertificateOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Button, Card, Col, Descriptions, Flex, Form, Input, Row, Space, Typography } from "antd";
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

    const onLogin = async ({ username, password }: LoginPayload) => {
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
        <main className="flex min-h-[100svh] flex-col p-6">
            <Flex align="center" justify="space-between">
                <Space>
                    <img
                        src={rustzenLogoUrl}
                        alt={APP_BRAND_NAME}
                        className="size-8 object-contain"
                    />
                    <Typography.Text strong>{APP_BRAND_NAME}</Typography.Text>
                </Space>
                <Space size="small">
                    <LanguageSwitch />
                    <ThemeSwitch />
                </Space>
            </Flex>

            <Row className="mx-auto w-full max-w-5xl flex-1" align="middle" gutter={[48, 32]}>
                <Col xs={24} lg={13}>
                    <Space orientation="vertical" size="large">
                        <div>
                            <Typography.Title>
                                {t(
                                    "统一运维，独立运行",
                                    "Unified operations, independent runtimes",
                                )}
                            </Typography.Title>
                            <Typography.Paragraph type="secondary">
                                {t(
                                    "通过一个管理入口查看 Admin、Monitor、Insights 和 Reports，同时保留各模块独立的进程、数据与故障边界。",
                                    "Manage Admin, Monitor, Insights, and Reports from one entry point while preserving independent processes, data, and failure boundaries.",
                                )}
                            </Typography.Paragraph>
                        </div>
                        <Descriptions
                            column={1}
                            items={[
                                {
                                    key: "modules",
                                    label: (
                                        <Space>
                                            <AppstoreOutlined />
                                            {t("模块", "Modules")}
                                        </Space>
                                    ),
                                    children: t(
                                        "四个独立运行模块",
                                        "Four independent runtime modules",
                                    ),
                                },
                                {
                                    key: "security",
                                    label: (
                                        <Space>
                                            <SafetyCertificateOutlined />
                                            {t("访问", "Access")}
                                        </Space>
                                    ),
                                    children: t(
                                        "统一认证与权限控制",
                                        "Unified authentication and access control",
                                    ),
                                },
                            ]}
                        />
                    </Space>
                </Col>

                <Col xs={24} lg={11}>
                    <Card aria-label={t("登录", "Sign in")}>
                        <Typography.Title level={2}>{t("登录", "Sign in")}</Typography.Title>
                        <Typography.Paragraph type="secondary">
                            {t(
                                "登录统一运维管理平台",
                                "Sign in to the operations management platform",
                            )}
                        </Typography.Paragraph>
                        <Form<LoginPayload>
                            layout="vertical"
                            autoComplete="off"
                            onFinish={onLogin}
                            requiredMark={false}
                        >
                            <Form.Item<LoginPayload>
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
                            <Form.Item<LoginPayload>
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
                                    {t("登录", "Sign in")}
                                </Button>
                            </Form.Item>
                        </Form>
                    </Card>
                </Col>
            </Row>
        </main>
    );
}
