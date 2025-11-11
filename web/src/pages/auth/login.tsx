import { LockOutlined, UserOutlined } from '@ant-design/icons';
import { useNavigate } from '@tanstack/react-router';
import { Button, Card, Form, Input, Typography } from 'antd';
import { useTransition } from 'react';

import { authAPI } from '@/api/auth';

import { useAuthStore } from '../../stores/useAuthStore';

export default function LoginPage() {
  console.log('LoginPage');
  const navigate = useNavigate();
  const [isPending, startTransition] = useTransition();
  const { handleLogin } = useAuthStore();
  const onLogin = async (values: Auth.LoginRequest) => {
    startTransition(async () => {
      try {
        const res = await authAPI.login(values);
        handleLogin(res.token, res.userInfo);
        navigate({ to: '/', replace: true });
      } catch (error) {
        console.error('Login failed', error);
      }
    });
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-50">
      <Card className="w-96">
        <div className="mb-8 text-center">
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
            username: 'admin',
            password: 'rustzen@2025',
          }}
        >
          <Form.Item
            name="username"
            rules={[
              {
                required: true,
                message: 'Please enter your username',
              },
              {
                min: 3,
                message: 'Username must be at least 3 characters',
              },
            ]}
          >
            <Input prefix={<UserOutlined />} placeholder="Username" />
          </Form.Item>
          <Form.Item
            name="password"
            rules={[
              {
                required: true,
                message: 'Please enter your password',
              },
              {
                min: 6,
                message: 'Password must be at least 6 characters',
              },
            ]}
          >
            <Input.Password prefix={<LockOutlined />} placeholder="Password" />
          </Form.Item>
          <Form.Item>
            <Button type="primary" htmlType="submit" loading={isPending} className="w-full">
              Login
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
}
