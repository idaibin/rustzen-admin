import "antd/dist/reset.css";
import "@ant-design/v5-patch-for-react-19";
import "@/index.css";

import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router-dom";
import { SWRConfig } from "swr";
import { App, ConfigProvider } from "antd";
import enUS from "antd/locale/en_US";
import type { useAppProps } from "antd/es/app/context";

import { swrFetcher } from "@/api";
import { router } from "@/router";

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
