import { EllipsisOutlined } from "@ant-design/icons";
import { forwardRef, useMemo, type ReactElement, type ReactNode } from "react";
import { Button, Dropdown, Tooltip, type ButtonProps, type DropdownProps } from "antd";

import { useAuthStore } from "@/store/useAuthStore";

const TABLE_ACTION_MIN_CONTENT_WIDTH = 32;
const TABLE_ACTION_CELL_PADDING = 32;
const TABLE_ACTION_ICON_WIDTH = 24;
const TABLE_ACTION_ICON_GAP = 8;

export const tableActionColumnWidth = (iconCount: number) => {
    const normalizedIconCount = Math.max(1, iconCount);
    const iconWidth =
        normalizedIconCount * TABLE_ACTION_ICON_WIDTH +
        (normalizedIconCount - 1) * TABLE_ACTION_ICON_GAP;

    return Math.max(TABLE_ACTION_MIN_CONTENT_WIDTH, iconWidth + TABLE_ACTION_CELL_PADDING);
};

interface TableActionButtonProps extends Omit<ButtonProps, "children" | "icon"> {
    label: string;
    icon: ReactNode;
}

export const TableActionButton = forwardRef<
    HTMLAnchorElement | HTMLButtonElement,
    TableActionButtonProps
>(({ label, icon, size = "small", variant = "text", htmlType = "button", ...props }, ref) => (
    <Tooltip title={label}>
        <Button
            ref={ref}
            aria-label={label}
            icon={icon}
            size={size}
            variant={variant}
            htmlType={htmlType}
            {...props}
        />
    </Tooltip>
));

TableActionButton.displayName = "TableActionButton";

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
