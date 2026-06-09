import { EllipsisOutlined } from "@ant-design/icons";
import { useMemo, type ReactElement, type ReactNode } from "react";
import { Button, Dropdown, Tooltip, type ButtonProps, type DropdownProps } from "antd";

import { useAuthStore } from "@/store/useAuthStore";

const TABLE_ACTION_ICON_SIZE = 24;
export const TABLE_ACTION_SPACE_SIZE = 4;

interface TableActionButtonProps extends Omit<ButtonProps, "children" | "icon"> {
    label: string;
    icon: ReactNode;
}

export const TableActionButton = ({
    label,
    icon,
    size = "small",
    variant = "text",
    htmlType = "button",
    style,
    ...props
}: TableActionButtonProps) => (
    <Tooltip title={label}>
        <Button
            aria-label={label}
            icon={icon}
            size={size}
            variant={variant}
            htmlType={htmlType}
            style={{
                minWidth: TABLE_ACTION_ICON_SIZE,
                width: TABLE_ACTION_ICON_SIZE,
                paddingInline: 0,
                ...style,
            }}
            {...props}
        />
    </Tooltip>
);

interface MoreButtonProps {
    children: ReactElement[];
    placement?: DropdownProps["placement"];
    trigger?: DropdownProps["trigger"];
    label?: string;
}

export const MoreButton = ({ children, ...props }: MoreButtonProps) => {
    const label = props.label ?? "More actions";
    const permissionSignature = useAuthStore((state) => state.userInfo?.permissions?.join("|") || "");
    const checkPermissions = useAuthStore((state) => state.checkPermissions);
    const items = useMemo(
        () =>
            children
                .filter((child) => {
                    const item = child.props as { code?: string; hidden?: boolean };
                    if (item.hidden) {
                        return false;
                    }
                    if (item.code) {
                        return checkPermissions(item.code);
                    }
                    return true;
                })
                .map((child, index) => ({
                    key: child?.key || index,
                    label: child,
                })),
        [children, checkPermissions, permissionSignature],
    );

    if (items.length === 0) {
        return null;
    }
    return (
        <Dropdown placement={props.placement} trigger={props.trigger} menu={{ items }}>
            <TableActionButton label={label} icon={<EllipsisOutlined />} />
        </Dropdown>
    );
};
