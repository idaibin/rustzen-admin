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

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <ConfigProvider locale={enUS}>
            <App>
                <SWRConfig value={{ fetcher: swrFetcher }}>
                    <RouterProvider router={router} />
                </SWRConfig>
            </App>
        </ConfigProvider>
    </React.StrictMode>
);
