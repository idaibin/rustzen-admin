import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { LoginResponse, UserInfoResponse } from "Auth";

interface AuthState {
  userInfo: UserInfoResponse | null;
  token: string | null;
  updateUserInfo: (params: UserInfoResponse) => void;
  updateToken: (params: string) => void;
  setAuth: (params: LoginResponse) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      userInfo: null,
      token: null,
      updateUserInfo: (params: UserInfoResponse) => {
        set({ userInfo: params });
      },
      updateToken: (params: string) => {
        set({ token: params });
      },
      setAuth: (params: LoginResponse) => {
        console.log("setAuth", params);
        set({ userInfo: params.userInfo, token: params.token });
        setTimeout(() => {
          console.log("setAuth timeout", get().userInfo);
          console.log("setAuth timeout", get().token);
        }, 2000);
      },
      // Clear all auth state
      clearAuth: () => {
        console.log("clearAuth");
        set({
          userInfo: null,
          token: null,
        });
      },
    }),
    {
      name: "auth-store",
    }
  )
);
