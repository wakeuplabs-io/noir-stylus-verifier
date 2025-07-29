import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn, shortenAddress } from "@/lib/utils";
import { createFileRoute, Link } from "@tanstack/react-router";
import {
  ArrowLeftIcon,
  CheckIcon,
  HelpCircleIcon,
  MinusIcon,
  MousePointer2Icon,
  MoveLeftIcon,
  PlusIcon,
  RadioIcon,
  SendHorizonal,
  SendHorizonalIcon,
  SendIcon,
  SquareChartGanttIcon,
  Users,
  UsersIcon,
  XIcon,
} from "lucide-react";
import Markdown from "react-markdown";
import { useState } from "react";
import { BackButton } from "@/components/back-button";

export const Route = createFileRoute("/proposals/new")({
  component: Index,
});

function Index() {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [preview, setPreview] = useState(false);

  return (
    <div className="">
      <div className="flex border-b items-center justify-between h-[72px] px-6">
        <BackButton to="/" />

        <button className="flex items-center gap-2 rounded-full border h-[46px] px-4 shrink-0 bg-black text-white">
          <span>Publish</span>
          <SendHorizonalIcon className="w-4 h-4" />
        </button>
      </div>

      <div className="flex divide-x min-h-screen">
        <div className="p-6 pb-20 w-full">
          <div className="bg-muted p-4 rounded-lg relative">
            <span className="text-xs text-muted-foreground absolute left-4 top-2">
              Title
            </span>
            <input
              type="text"
              className="w-full h-full outline-none mt-4"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </div>

          <div className="mt-5">
            <div className="flex gap-2  mb-3 font-bold">
              <button
                className={cn(
                  "uppercase text-sm",
                  preview ? "text-muted-foreground" : ""
                )}
                onClick={() => setPreview(false)}
              >
                Write
              </button>
              <button
                className={cn(
                  "uppercase text-sm",
                  !preview ? "text-muted-foreground" : ""
                )}
                onClick={() => setPreview(true)}
              >
                Preview
              </button>
            </div>

            {preview ? (
              <div className="prose border rounded-lg p-4">
                <Markdown>{body}</Markdown>
              </div>
            ) : (
              <textarea
                value={body}
                onChange={(e) => setBody(e.target.value)}
                className="w-full h-full outline-none bg-muted p-4 rounded-lg"
                placeholder="Title"
                rows={10}
              />
            )}
          </div>
        </div>

        <div className="w-[600px] p-6 pb-20">
          <div className="flex items-center gap-2 mb-3 text-gray-800">
            <UsersIcon className="h-4 w-4" />

            <span className="uppercase font-semibold text-sm">Voters</span>
          </div>

          <div className="border rounded-lg">
            <div className="p-4 border-b">
              <input
                type="text"
                className="w-full outline-none text-sm"
                placeholder="Input ZK address"
              />
            </div>

            {/* <div className="p-4 text-sm">
                No voters yet
              </div> */}
            <div className="p-4 text-sm">
              <div className="flex justify-between items-center">
                <span>
                  {shortenAddress("0x9D39B627E6769B0b77f03825C118Ec48c84A8fbD")}
                </span>
                <button>
                  <XIcon className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
