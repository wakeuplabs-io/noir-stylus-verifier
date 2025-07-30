import { cn, shortenAddress } from "@/lib/utils";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { SendHorizontalIcon, UsersIcon, XIcon } from "lucide-react";
import Markdown from "react-markdown";
import { useState, useMemo } from "react";
import { BackButton } from "@/components/back-button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import { ChevronDownIcon } from "lucide-react";
import { toast } from "sonner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useCreateProposal } from "@/lib/queries/proposal";
import { proposalMetadataSchema, zkAddressSchema } from "@voting/core";

export const Route = createFileRoute("/proposals/new")({
  component: Index,
});

function Index() {
  const navigate = useNavigate();

  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [preview, setPreview] = useState(false);
  const [calendarOpen, setCalendarOpen] = useState(false);
  const [deadline, setDeadline] = useState<Date>(new Date());
  const [address, setAddress] = useState("");
  const [voters, setVoters] = useState<string[]>([]);

  const { mutateAsync: createProposal, isPending: isCreatingProposal } =
    useCreateProposal();

  const onAddVoter = (address: string) => {
    const result = zkAddressSchema.safeParse(address);

    if (!result.success) {
      toast.error(result.error.issues[0].message);
      return;
    }

    if (voters.includes(address)) {
      toast.error("Voter already added");
      return;
    }

    setVoters([...voters, address]);
    setAddress("");
  };

  const onRemoveVoter = (address: string) => {
    setVoters(voters.filter((voter) => voter !== address));
  };

  const onCreateProposal = async () => {
    // submit is disabled if validation fails, so here we can assume it's valid
    try {
      const proposalId = await createProposal({
        title,
        body,
        deadline,
        voters,
      });

      navigate({ to: "/proposals/$id", params: { id: proposalId.toString() } });
    } catch (error) {
      console.error(error);
      toast.error("Failed to create proposal");
    }
  };

  const validation = useMemo(() => {
    const result = proposalMetadataSchema.safeParse({
      title,
      body,
      deadline,
      voters,
    });

    if (result.success) {
      return { isValid: true, errors: [] };
    }

    const errors = result.error.issues.map((issue) => issue.message);
    return { isValid: false, errors };
  }, [title, body, deadline, voters]);

  const errorMessage = validation.errors[0];

  return (
    <div className="">
      <div className="flex border-b items-center justify-between h-[72px] px-6">
        <BackButton to="/" />

        <Tooltip>
          <TooltipTrigger asChild>
            <button
              disabled={!validation.isValid || isCreatingProposal}
              onClick={onCreateProposal}
              className="flex items-center gap-2 rounded-full border h-[46px] px-4 shrink-0 bg-black text-white disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span>{isCreatingProposal ? "Creating..." : "Publish"}</span>
              <SendHorizontalIcon className="w-4 h-4" />
            </button>
          </TooltipTrigger>
          {!validation.isValid && (
            <TooltipContent side="bottom" className="max-w-[300px]">
              {errorMessage}
            </TooltipContent>
          )}
        </Tooltip>
      </div>

      <div className="flex divide-x min-h-screen">
        <div className="p-6 pb-20 w-full">
          {/* Title input */}
          <div className="bg-muted p-4 rounded-lg relative mb-4">
            <span className="text-xs text-muted-foreground absolute left-4 top-2">
              Title
            </span>
            <input
              type="text"
              value={title}
              placeholder="Proposal title"
              className="w-full h-full outline-none mt-4"
              onChange={(e) => setTitle(e.target.value)}
            />
          </div>

          {/* Deadline */}
          <div className="mb-6">
            <div className="flex gap-4">
              <Popover open={calendarOpen} onOpenChange={setCalendarOpen}>
                <PopoverTrigger asChild>
                  <button
                    id="date-picker"
                    className="bg-muted p-4 rounded-lg relative min-w-[200px] w-full"
                  >
                    <span className="text-xs text-muted-foreground absolute left-4 top-2">
                      Deadline date
                    </span>
                    <div className="flex items-center justify-between mt-4">
                      {deadline ? deadline.toDateString() : "Select date"}
                      <ChevronDownIcon className="w-4 h-4" />
                    </div>
                  </button>
                </PopoverTrigger>
                <PopoverContent
                  className="w-auto overflow-hidden p-0"
                  align="start"
                >
                  <Calendar
                    mode="single"
                    selected={deadline}
                    captionLayout="dropdown"
                    onSelect={(date) => {
                      if (date) {
                        setDeadline(date);
                        setCalendarOpen(false);
                      }
                    }}
                  />
                </PopoverContent>
              </Popover>
              <div className="bg-muted p-4 rounded-lg relative">
                <span className="text-xs text-muted-foreground absolute left-4 top-2">
                  Deadline time
                </span>
                <input
                  type="time"
                  id="time-picker"
                  step="1"
                  defaultValue="10:30:00"
                  className="w-full outline-none mt-4 appearance-none [&::-webkit-calendar-picker-indicator]:hidden [&::-webkit-calendar-picker-indicator]:appearance-none"
                />
              </div>
            </div>
          </div>

          {/* Body input */}
          <div className="mb-6">
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
                placeholder="Write your markdown proposal here..."
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
            <form
              onSubmit={(e) => {
                e.preventDefault();
                onAddVoter(address);
              }}
              className="p-4 border-b"
            >
              <input
                type="text"
                className="w-full outline-none text-sm"
                placeholder="Input ZK address"
                value={address}
                onChange={(e) => setAddress(e.target.value)}
              />
            </form>

            <div className="text-sm divide-y">
              {voters.map((voter) => (
                <div
                  key={voter}
                  className="flex justify-between items-center px-4 py-3"
                >
                  <span>{shortenAddress(voter)}</span>
                  <button onClick={() => onRemoveVoter(voter)}>
                    <XIcon className="w-4 h-4" />
                  </button>
                </div>
              ))}

              {voters.length === 0 && (
                <div className="text-sm text-muted-foreground px-4 py-3">
                  Add at least one voter to publish your proposal
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
