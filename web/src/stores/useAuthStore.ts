import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface AuthState {
  userInfo: Auth.UserInfoResponse | null;
  token: string | null;
  handleLogin: (token: string, userInfo: Auth.UserInfoResponse) => void;
  updateToken: (params: string) => void;
  updateAvatar: (avatarUrl: string) => void;
  updateUserInfo: (params: Auth.UserInfoResponse) => void;
  clearAuth: () => void;
  checkPermissions: (code: string) => boolean;
  checkMenuPermissions: (path: string) => boolean;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      userInfo: null,
      token: null,
      handleLogin: (token, userInfo) => {
        set({ token, userInfo });
      },
      updateToken: (params: string) => {
        set({ token: params });
      },
      updateAvatar: (avatarUrl: string) => {
        set({
          userInfo: {
            ...(get().userInfo as Auth.UserInfoResponse),
            avatarUrl,
          },
        });
      },
      updateUserInfo: (params: Auth.UserInfoResponse) => {
        set({ userInfo: params });
      },

      // Clear all auth state
      clearAuth: () => {
        set({ userInfo: null, token: null });
      },
      checkPermissions: (code: string) => {
        const permissions = get().userInfo?.permissions || [];
        if (permissions.length === 0) {
          return false;
        }
        if (permissions.includes('*')) {
          return true;
        }
        if (permissions.includes(code)) {
          return true;
        }
        // system:user:* -> system:*
        const codeArr = code.split(':');
        for (let i = codeArr.length - 1; i > 0; i--) {
          const prefix = codeArr.slice(0, i).join(':') + ':*';
          if (permissions.includes(prefix)) {
            return true;
          }
        }
        return false;
      },
      checkMenuPermissions: (path: string) => {
        const code = formatPathCode(path);
        return get().checkPermissions(code);
      },
    }),
    {
      name: 'auth-store',
    },
  ),
);

const formatPathCode = (pathname: string) => {
  const code = pathname.replace(/\//g, ':').slice(1);
  // create page
  if (code.endsWith(':create')) {
    return code;
  }
  // edit page, detail page
  if (code.endsWith(':edit') || code.endsWith(':detail')) {
    return code
      .split(':')
      .filter((s) => !/^\d+$/.test(s))
      .join(':');
  }
  // list page
  return `${code}:list`;
};
