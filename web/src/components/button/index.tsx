import { Dropdown, type DropdownProps } from 'antd';

import { useAuthStore } from '@/stores/useAuthStore';

interface MoreButtonProps {
  children: React.ReactElement[];
  placement?: DropdownProps['placement'];
  trigger?: DropdownProps['trigger'];
}

export const MoreButton = ({ children, ...props }: MoreButtonProps) => {
  const items = children
    .filter((child) => {
      const item = child.props as { code?: string; hidden?: boolean };
      if (item.hidden) {
        return false;
      }
      if (item.code) {
        return useAuthStore.getState().checkPermissions(item.code);
      }
      return true;
    })
    .map((child, index) => ({
      key: child?.key || index,
      label: child,
    }));

  if (items.length === 0) {
    return null;
  }
  return (
    <Dropdown placement={props.placement} trigger={props.trigger} menu={{ items }}>
      <a>More</a>
    </Dropdown>
  );
};
