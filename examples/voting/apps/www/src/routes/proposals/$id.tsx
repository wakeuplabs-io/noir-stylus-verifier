import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/proposals/$id')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/proposals/$id"!</div>
}
