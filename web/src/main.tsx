import "antd/dist/reset.css";
import "./index.css";
import "@ant-design/v5-patch-for-react-19";
import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router-dom";
import { SWRConfig } from "swr";
import { swrFetcher } from "./api/request";
import { router } from "./router";
import { App, ConfigProvider } from "antd";
import enUS from "antd/locale/en_US";
import type { useAppProps } from "antd/es/app/context";

// 初始化通用提示
export let messageApi: useAppProps["message"];
export let notificationApi: useAppProps["notification"];
export let modalApi: useAppProps["modal"];

const InitMethods = () => {
    const { message, notification, modal } = App.useApp();
    messageApi = message;
    notificationApi = notification;
    modalApi = modal;
    return null;
};

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <ConfigProvider locale={enUS}>
            <App>
                <InitMethods />
                <SWRConfig value={{ fetcher: swrFetcher }}>
                    <RouterProvider router={router} />
                </SWRConfig>
            </App>
        </ConfigProvider>
    </React.StrictMode>
);
