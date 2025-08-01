import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/')({
  component: Index,
})

function Index() {
  return (
    <div className="p-2 bg-black h-screen w-screen flex justify-center items-center">
      <h1 className="text-white text-2xl font-bold">Battleship</h1>
    </div>
  )
}