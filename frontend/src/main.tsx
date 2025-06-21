import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router-dom";
import { SWRConfig } from "swr";
import { swrFetcher } from "./services/api";
import { router } from "./router";
import "antd/dist/reset.css";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <SWRConfig
      value={{
        fetcher: swrFetcher,
      }}
    >
      <RouterProvider router={router} />
    </SWRConfig>
  </React.StrictMode>
);
