import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/translate/')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/translate/"!</div>
}
