import React, { useState } from "react";
import { Form, Input, Button, Card, Typography } from "antd";
import { UserOutlined, LockOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../../stores/useAuthStore";
import type { LoginRequest } from "Auth";
import { authAPI } from "@/services";

const { Title, Text } = Typography;

const LoginPage: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const { setAuth } = useAuthStore();

  const onLogin = async (values: LoginRequest) => {
    setLoading(true);
    try {
      const res = await authAPI.login(values);
      setAuth(res);
      navigate("/", { replace: true });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <Card className="w-full max-w-md">
        <div className="text-center mb-8">
          <Title level={2} className="mb-2">
            Login
          </Title>
          <Text type="secondary">Please sign in to your account</Text>
        </div>
        <Form
          name="login"
          onFinish={onLogin}
          autoComplete="off"
          size="large"
          initialValues={{ username: "zenadmin", password: "ZenAdmin@4321" }}
        >
          <Form.Item
            name="username"
            rules={[
              { required: true, message: "Please enter your username" },
              { min: 3, message: "Username must be at least 3 characters" },
            ]}
          >
            <Input prefix={<UserOutlined />} placeholder="Username" />
          </Form.Item>
          <Form.Item
            name="password"
            rules={[
              { required: true, message: "Please enter your password" },
              { min: 6, message: "Password must be at least 6 characters" },
            ]}
          >
            <Input.Password prefix={<LockOutlined />} placeholder="Password" />
          </Form.Item>
          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              loading={loading}
              className="w-full"
            >
              Login
            </Button>
          </Form.Item>
          <div className="text-center">
            <Text type="secondary">
              Don't have an account?{" "}
              <a
                onClick={() => navigate("/register")}
                className="text-blue-500 cursor-pointer"
              >
                Register here
              </a>
            </Text>
          </div>
        </Form>
      </Card>
    </div>
  );
};

export default LoginPage;
