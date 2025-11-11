import { createFileRoute } from '@tanstack/react-router';

import MenuPage from '@/pages/system/menu';

export const Route = createFileRoute('/system/menu')({
  component: MenuPage,
});
