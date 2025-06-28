import "antd/dist/reset.css";
import "./index.css";
import "@ant-design/v5-patch-for-react-19";
import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router-dom";
import { SWRConfig } from "swr";
import { swrFetcher } from "./services/api";
import { router } from "./router";
import { App } from "antd";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App>
      <SWRConfig
        value={{
          fetcher: swrFetcher,
        }}
      >
        <RouterProvider router={router} />
      </SWRConfig>
    </App>
  </React.StrictMode>
);
