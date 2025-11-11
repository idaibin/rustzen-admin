import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/403')({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/error/403"!</div>;
}
