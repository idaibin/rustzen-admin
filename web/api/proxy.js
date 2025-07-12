import { createProxyMiddleware } from "http-proxy-middleware";

// 只导出一个 default 函数即可
export default function handler(req, res) {
    const proxy = createProxyMiddleware({
        target: process.env.API_TARGET, // 设置环境变量：真实后端地址
        changeOrigin: true,
        pathRewrite: {},
    });
    return proxy(req, res);
}
