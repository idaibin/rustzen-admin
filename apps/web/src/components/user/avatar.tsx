import { UploadOutlined } from "@ant-design/icons";
import { Avatar, Button, Upload, type UploadProps } from "antd";
import { useState } from "react";

import { accountAPI, appMessage } from "@/api";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

const MAX_AVATAR_SIZE = 1024 * 1024;
const ALLOWED_AVATAR_TYPES = new Set(["image/jpeg", "image/png"]);

export const UserAvatar = () => {
    const { userInfo, updateAvatar } = useAuthStore();
    const [uploading, setUploading] = useState(false);

    const validateFile = (file: File) => {
        if (!ALLOWED_AVATAR_TYPES.has(file.type)) {
            appMessage.error(
                t("只能上传 JPG/JPEG/PNG 文件！", "Only JPG, JPEG, and PNG files are supported."),
            );
            return false;
        }
        if (file.size > MAX_AVATAR_SIZE) {
            appMessage.error(t("图片必须小于 1MB！", "The image must be smaller than 1 MB."));
            return false;
        }

        return true;
    };

    const handleUpload: UploadProps["customRequest"] = async ({ file, onSuccess, onError }) => {
        const rawFile = file as Blob & { originFileObj?: File };
        const uploadFile = rawFile instanceof File ? rawFile : rawFile.originFileObj;
        if (!(uploadFile instanceof File)) {
            onError?.(new Error("invalid-file"));
            return;
        }

        if (!validateFile(uploadFile)) {
            onError?.(new Error("invalid-file"));
            return;
        }

        setUploading(true);
        try {
            const avatarUrl = await accountAPI.updateAvatar({ file: uploadFile });
            updateAvatar(avatarUrl);
            appMessage.success(t("头像已更新", "Avatar updated"));
            onSuccess?.("ok");
        } catch (error) {
            appMessage.error(
                error instanceof Error ? error.message : t("上传失败", "Upload failed"),
            );
            onError?.(new Error(t("上传失败", "Upload failed")));
        } finally {
            setUploading(false);
        }
    };

    return (
        <div className="flex w-full flex-col items-center gap-3 text-center">
            <Avatar size={96} src={userInfo?.avatarUrl} className="border">
                {avatarFallback(userInfo)}
            </Avatar>
            <Upload
                accept="image/png,image/jpeg"
                showUploadList={false}
                customRequest={handleUpload}
                disabled={uploading}
            >
                <Button type="default" icon={<UploadOutlined />} loading={uploading}>
                    {uploading ? t("正在上传", "Uploading") : t("上传头像", "Upload avatar")}
                </Button>
            </Upload>
            <div className="text-sm text-muted-foreground">
                <div>{t("格式：JPG、PNG、JPEG", "Format: JPG, PNG, JPEG")}</div>
                <div>{t("大小：小于 1 MB", "Size: under 1 MB")}</div>
            </div>
        </div>
    );
};

const avatarFallback = (userInfo: Auth.UserInfoResponse | null) => {
    const displayName = userInfo?.realName || userInfo?.username || "RA";
    return displayName.slice(0, 2).toUpperCase();
};
