import { createFileRoute } from '@tanstack/react-router';

import RolePage from '@/pages/system/role';

export const Route = createFileRoute('/system/role')({
  component: RolePage,
});
