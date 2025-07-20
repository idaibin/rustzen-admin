import React, { useEffect } from "react";
import { Navigate, useLocation } from "react-router-dom";
import { useAuthStore } from "@/stores/useAuthStore";
import { authAPI } from "@/services";
import useSWR from "swr";
import type { UserInfoResponse } from "Auth";

interface AuthGuardProps {
    children: React.ReactNode;
}

export const AuthGuard: React.FC<AuthGuardProps> = ({ children }) => {
    const location = useLocation();
    const { token, updateUserInfo, checkPermision } = useAuthStore();
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

    const code = location.pathname.replace(/\//g, ":").slice(1);
    const isPermision = checkPermision(code, true);
    if (location.pathname !== "/" && !isPermision) {
        return <Navigate to="/403" replace />;
    }

    // Render children if authenticated
    return children;
};

interface AuthWrapProps {
    code: string;
    children: React.ReactNode;
    hidden?: boolean;
}

export const AuthWrap: React.FC<AuthWrapProps> = ({
    code,
    children,
    hidden = false,
}) => {
    const { checkPermision } = useAuthStore();
    const isPermision = checkPermision(code, false);
    if (isPermision && !hidden) {
        return children;
    }
    return null;
};
