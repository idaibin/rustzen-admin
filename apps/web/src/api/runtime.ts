import { App } from "antd";
import { useEffect } from "react";

type MessageContent = string | Error;

const formatMessage = (message: MessageContent) => {
    return message instanceof Error ? message.message : message;
};

type MessageApi = {
    success: (content: string) => void;
    error: (content: string) => void;
    info: (content: string) => void;
    warning: (content: string) => void;
    loading: (content: string) => void;
};

const fallbackMessage: MessageApi = {
    success: (content: string) => {
        console.info(content);
    },
    error: (content: string) => {
        console.error(content);
    },
    info: (content: string) => {
        console.info(content);
    },
    warning: (content: string) => {
        console.warn(content);
    },
    loading: (content: string) => {
        console.info(content);
    },
};

let messageApi: MessageApi = fallbackMessage;

export const appMessage = {
    success: (message: MessageContent) => messageApi.success(formatMessage(message)),
    error: (message: MessageContent) => messageApi.error(formatMessage(message)),
    info: (message: MessageContent) => messageApi.info(formatMessage(message)),
    warning: (message: MessageContent) => messageApi.warning(formatMessage(message)),
    loading: (message: MessageContent) => messageApi.loading(formatMessage(message)),
};

export const MessageContent = () => {
    const { message } = App.useApp();

    useEffect(() => {
        const nextApi = {
            success: (content: string) => message.success(content),
            error: (content: string) => message.error(content),
            info: (content: string) => message.info(content),
            warning: (content: string) => message.warning(content),
            loading: (content: string) => message.loading(content),
        };

        messageApi = nextApi;
        return () => {
            messageApi = fallbackMessage;
        };
    }, [message]);

    return null;
};
