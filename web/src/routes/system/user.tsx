import { createFileRoute } from '@tanstack/react-router';

import UserPage from '@/pages/system/user';

export const Route = createFileRoute('/system/user')({
  component: UserPage,
});
