import { App } from "antd";
import type { MessageInstance } from "antd/es/message/interface";
import type { ModalStaticFunctions } from "antd/es/modal/confirm";

let appMessage: MessageInstance;
let appModal: Omit<ModalStaticFunctions, "warn">;

export const MessageContent = () => {
    const staticFunction = App.useApp();
    appMessage = staticFunction.message;
    appModal = staticFunction.modal;
    return null;
};

export { appMessage, appModal };
