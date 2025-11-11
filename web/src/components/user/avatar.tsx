import { UploadOutlined } from '@ant-design/icons';
import { Upload, type UploadFile } from 'antd';

import { appMessage } from '@/api';
import { useAuthStore } from '@/stores/useAuthStore';

// const getBase64 = (img: UploadFile, callback: (url: string) => void) => {
//     const reader = new FileReader();
//     reader.addEventListener("load", () => callback(reader.result as string));
//     reader.readAsDataURL(img.originFileObj as Blob);
// };

const beforeUpload = async (file: UploadFile) => {
  if (!file.size) {
    return false;
  }
  const isJpgOrPng = file.type === 'image/jpeg' || file.type === 'image/png';
  if (!isJpgOrPng) {
    appMessage.error('You can only upload JPG/JPEG/PNG file!');
    return false;
  }
  const isLimt = file.size / 1024 / 1024 <= 1;
  if (!isLimt) {
    appMessage.error('Image must smaller than 1MB!');
    return false;
  }
  return isJpgOrPng && isLimt;
};
export const UserAvatar = () => {
  const { userInfo, token, updateAvatar } = useAuthStore();

  return (
    <>
      <Upload
        accept="image/*"
        name="avatar"
        listType="picture-circle"
        showUploadList={false}
        action="/api/auth/avatar"
        beforeUpload={beforeUpload}
        headers={{
          Authorization: `Bearer ${token}`,
        }}
        onChange={(info) => {
          if (info.file.status === 'done') {
            updateAvatar(info.file.response.data);
          }
        }}
      >
        {userInfo?.avatarUrl ? (
          <img src={userInfo?.avatarUrl} className="rounded-full" alt="avatar" />
        ) : (
          <UploadOutlined />
        )}
      </Upload>
      <div className="mt-2 w-full text-center">上传头像</div>
      <div className="w-full text-left text-gray-500">格式：支持JPG、PNG、JPEG</div>
      <div className="w-full text-left text-gray-500">大小：1M以内</div>
    </>
  );
};
