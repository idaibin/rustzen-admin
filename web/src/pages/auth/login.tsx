import { useTransition } from "react";
import { Form, Input, Button, Card, Typography } from "antd";
import { UserOutlined, LockOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../../stores/useAuthStore";
import type { LoginRequest } from "Auth";
import { authAPI } from "@/api";

export default function LoginPage() {
    const navigate = useNavigate();
    const [isPending, startTransition] = useTransition();
    const { updateToken } = useAuthStore();
    const onLogin = async (values: LoginRequest) => {
        startTransition(async () => {
            try {
                const res = await authAPI.login(values);
                updateToken(res.token);
                navigate("/", { replace: true });
            } catch (error) {
                console.error("Login failed", error);
            }
        });
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50">
            <Card className="w-96">
                <div className="text-center mb-8">
                    <Typography.Title level={2} className="mb-2">
                        Rustzen Admin
                    </Typography.Title>
                </div>
                <Form
                    name="login"
                    onFinish={onLogin}
                    autoComplete="off"
                    size="large"
                    initialValues={{
                        username: "superadmin",
                        password: "rustzen@123",
                    }}
                >
                    <Form.Item
                        name="username"
                        rules={[
                            {
                                required: true,
                                message: "Please enter your username",
                            },
                            {
                                min: 3,
                                message:
                                    "Username must be at least 3 characters",
                            },
                        ]}
                    >
                        <Input
                            prefix={<UserOutlined />}
                            placeholder="Username"
                        />
                    </Form.Item>
                    <Form.Item
                        name="password"
                        rules={[
                            {
                                required: true,
                                message: "Please enter your password",
                            },
                            {
                                min: 6,
                                message:
                                    "Password must be at least 6 characters",
                            },
                        ]}
                    >
                        <Input.Password
                            prefix={<LockOutlined />}
                            placeholder="Password"
                        />
                    </Form.Item>
                    <Form.Item>
                        <Button
                            type="primary"
                            htmlType="submit"
                            loading={isPending}
                            className="w-full"
                        >
                            Login
                        </Button>
                    </Form.Item>
                </Form>
            </Card>
        </div>
    );
}
