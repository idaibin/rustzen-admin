import React, { useEffect } from "react";
import { Navigate, useLocation } from "react-router-dom";
import { useAuthStore } from "../stores/useAuthStore";
import { authAPI } from "@/services";
import useSWR from "swr";
import type { UserInfoResponse } from "Auth";

interface AuthGuardProps {
  children: React.ReactNode;
}

const AuthGuard: React.FC<AuthGuardProps> = ({ children }) => {
  const location = useLocation();
  const { token, updateUserInfo } = useAuthStore();
  const { data: userInfo } = useSWR<UserInfoResponse>(
    authAPI.urls.getUserInfo()
  );

  useEffect(() => {
    if (userInfo !== undefined) {
      updateUserInfo(userInfo);
    }
  }, [userInfo, updateUserInfo]);

  // Redirect to login if no token
  if (!token) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  // Render children if authenticated
  return <>{children}</>;
};

export default AuthGuard;
