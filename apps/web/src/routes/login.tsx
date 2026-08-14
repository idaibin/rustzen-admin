import {
    BarChartOutlined,
    CloudServerOutlined,
    FileTextOutlined,
    LockOutlined,
    SafetyCertificateOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Button, Card, Form, Input, Typography, type FormProps } from "antd";
import { useState, type ReactNode } from "react";

import { authAPI } from "@/api";
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
            <div className="relative mx-auto min-h-[100svh] w-full max-w-[1440px] px-6 sm:px-10 lg:px-16">
                <header className="absolute inset-x-6 top-6 z-10 flex h-10 items-center gap-3 sm:inset-x-10 lg:inset-x-16">
                    <img
                        src={rustzenLogoUrl}
                        alt={APP_BRAND_NAME}
                        className="size-10 object-contain"
                    />
                    <span className="text-xl font-semibold leading-none">{APP_BRAND_NAME}</span>
                    <div className="ms-auto flex items-center gap-1">
                        <LanguageSwitch />
                        <ThemeSwitch />
                    </div>
                </header>

                <div className="grid min-h-[100svh] w-full items-center gap-16 py-24 lg:grid-cols-[minmax(0,1fr)_440px] xl:gap-28">
                    <section className="hidden max-w-2xl lg:block">
                        <Typography.Text className="mb-5 block text-sm font-semibold text-primary">
                            {t(
                                "轻量、自托管、边界清晰",
                                "Lightweight, self-hosted, clearly bounded",
                            )}
                        </Typography.Text>
                        <Typography.Title
                            level={1}
                            className="!mb-5 !text-4xl !leading-tight !tracking-tight xl:!text-5xl"
                        >
                            {t("一个入口，掌握系统运行状态", "One console for operational clarity")}
                        </Typography.Title>
                        <Typography.Paragraph className="!mb-10 max-w-xl !text-base !leading-7 !text-muted-foreground">
                            {t(
                                "统一管理账号、运行模块、基础监控、产品分析与自动化报表。",
                                "Manage accounts, runtime modules, monitoring, analytics, and automated reports in one place.",
                            )}
                        </Typography.Paragraph>
                        <div className="grid max-w-xl grid-cols-2 gap-x-10 gap-y-6">
                            <Capability
                                icon={<SafetyCertificateOutlined />}
                                label={t("身份与权限", "Identity and access")}
                            />
                            <Capability
                                icon={<CloudServerOutlined />}
                                label={t("节点与服务监控", "Node and service monitoring")}
                            />
                            <Capability
                                icon={<BarChartOutlined />}
                                label={t("产品行为分析", "Product analytics")}
                            />
                            <Capability
                                icon={<FileTextOutlined />}
                                label={t("自动化报表", "Automated reports")}
                            />
                        </div>
                    </section>

                    <Card
                        className="page-panel w-full max-w-110 justify-self-center bg-card"
                        styles={{ body: { padding: 0 } }}
                        role="region"
                        aria-label={t("登录", "Sign in")}
                    >
                        <div className="px-7 py-9 sm:px-10 sm:py-11">
                            <div className="mb-8">
                                <Typography.Title
                                    level={2}
                                    className="!mb-2 !text-2xl !font-semibold !tracking-tight"
                                >
                                    {t("登录 RustZen Admin", "Sign in to RustZen Admin")}
                                </Typography.Title>
                                <Typography.Text type="secondary">
                                    {t(
                                        "使用管理员账号继续",
                                        "Continue with your administrator account",
                                    )}
                                </Typography.Text>
                            </div>

                            <Form
                                layout="vertical"
                                autoComplete="off"
                                onFinish={onLogin}
                                className="grid gap-5"
                                requiredMark={false}
                            >
                                <div className="grid gap-2">
                                    <label
                                        className="text-sm font-medium text-foreground"
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
                                            size="large"
                                            className="shadow-none"
                                        />
                                    </Form.Item>
                                </div>

                                <div className="grid gap-2">
                                    <div className="text-sm leading-none">
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
                                            size="large"
                                            className="shadow-none"
                                        />
                                    </Form.Item>
                                </div>

                                <Form.Item className="!mb-0">
                                    <Button
                                        type="primary"
                                        htmlType="submit"
                                        block
                                        loading={isSubmitting}
                                        size="large"
                                        className="mt-1 font-medium shadow-none"
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

function Capability({ icon, label }: { icon: ReactNode; label: string }) {
    return (
        <div className="flex items-center gap-3 text-sm font-medium text-foreground">
            <span className="flex size-9 items-center justify-center rounded-lg bg-accent text-primary">
                {icon}
            </span>
            <span>{label}</span>
        </div>
    );
}
