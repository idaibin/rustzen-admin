import React, { useState } from "react";
import { Form, Input, Button, Card, Typography } from "antd";
import { UserOutlined, LockOutlined, MailOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { authAPI } from "@/services";

const { Title, Text } = Typography;

// Define the registration form type
interface RegisterForm {
  username: string;
  email: string;
  password: string;
  confirm: string;
}

const RegisterPage: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const onRegister = async (values: RegisterForm) => {
    setLoading(true);
    try {
      await authAPI.register(values);
      navigate("/login", { replace: true });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <Card className="w-full max-w-md">
        <div className="text-center mb-8">
          <Title level={2} className="mb-2">
            Create Account
          </Title>
          <Text type="secondary">Please fill in the information below</Text>
        </div>
        <Form
          name="register"
          onFinish={onRegister}
          autoComplete="off"
          size="large"
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
            name="email"
            rules={[
              { required: true, message: "Please enter your email" },
              { type: "email", message: "Please enter a valid email" },
            ]}
          >
            <Input prefix={<MailOutlined />} placeholder="Email" />
          </Form.Item>
          <Form.Item
            name="password"
            rules={[
              { required: true, message: "Please enter your password" },
              { min: 6, message: "Password must be at least 6 characters" },
            ]}
            hasFeedback
          >
            <Input.Password prefix={<LockOutlined />} placeholder="Password" />
          </Form.Item>
          <Form.Item
            name="confirm"
            dependencies={["password"]}
            hasFeedback
            rules={[
              { required: true, message: "Please confirm your password" },
              ({ getFieldValue }) => ({
                validator(_, value) {
                  if (!value || getFieldValue("password") === value) {
                    return Promise.resolve();
                  }
                  return Promise.reject(new Error("Passwords do not match!"));
                },
              }),
            ]}
          >
            <Input.Password
              prefix={<LockOutlined />}
              placeholder="Confirm Password"
            />
          </Form.Item>
          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              loading={loading}
              className="w-full"
            >
              Register
            </Button>
          </Form.Item>
          <div className="text-center">
            <Text type="secondary">
              Already have an account?{" "}
              <a
                onClick={() => navigate("/auth/login")}
                className="text-blue-500 cursor-pointer"
              >
                Login here
              </a>
            </Text>
          </div>
        </Form>
      </Card>
    </div>
  );
};

export default RegisterPage;
