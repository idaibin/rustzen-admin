import { createFileRoute } from '@tanstack/react-router';

import DictPage from '@/pages/system/dict';

export const Route = createFileRoute('/system/dict')({
  component: DictPage,
});
