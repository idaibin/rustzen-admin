import { createFileRoute } from '@tanstack/react-router';

import LogPage from '@/pages/system/log';

export const Route = createFileRoute('/system/log')({
  component: LogPage,
});
