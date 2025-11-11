import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/404')({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/error/404"!</div>;
}
