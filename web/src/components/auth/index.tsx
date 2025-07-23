import React, { useEffect } from "react";
import { Navigate, useLocation } from "react-router-dom";
import { useAuthStore } from "@/stores/useAuthStore";
import { authAPI } from "@/api";
import useSWR from "swr";

interface AuthGuardProps {
    children: React.ReactNode;
}

export const AuthGuard: React.FC<AuthGuardProps> = ({ children }) => {
    const location = useLocation();
    const { token, updateUserInfo, checkMenuPermissions } = useAuthStore();
    const { data: userInfo } = useSWR("getUserInfo", authAPI.getUserInfo);

    useEffect(() => {
        if (userInfo !== undefined) {
            updateUserInfo(userInfo);
        }
    }, [userInfo, updateUserInfo]);

    // Redirect to login if no token
    if (!token) {
        return <Navigate to="/login" state={{ from: location }} replace />;
    }
    if (location.pathname === "/") {
        return children;
    }

    const isPermission = checkMenuPermissions(location.pathname);

    return isPermission ? children : <Navigate to="/403" replace />;
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
    const { checkPermissions } = useAuthStore();
    const isPermission = checkPermissions(code);
    if (isPermission && !hidden) {
        return children;
    }
    return null;
};
