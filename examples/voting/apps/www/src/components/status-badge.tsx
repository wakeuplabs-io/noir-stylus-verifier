import { cn } from "@/lib/utils";
import type { ProposalStatus } from "@voting/core";
import { CheckIcon, RadioIcon, XIcon } from "lucide-react";

export const StatusIcon: React.FC<{
  status: ProposalStatus;
  className?: string;
}> = ({ status, className }) => {
  if (status === "active") {
    return (
      <RadioIcon
        className={cn("w-[18px] h-[18px] text-green-500", className)}
      />
    );
  }
  if (status === "passed") {
    return (
      <div
        className={cn(
          "w-[14px] h-[14px] rounded-full bg-gray-900 flex items-center justify-center",
          className
        )}
      >
        <CheckIcon className="w-2 h-2 text-white" />
      </div>
    );
  }
  if (status === "rejected") {
    return (
      <div
        className={cn(
          "w-[14px] h-[14px] rounded-full bg-gray-900 flex items-center justify-center",
          className
        )}
      >
        <XIcon className="w-2 h-2 text-white" />
      </div>
    );
  }
  return null;
};

export const StatusBadge: React.FC<{
  status: ProposalStatus;
  className?: string;
}> = ({ status, className }) => {
  return (
    <div
      className={cn(
        "flex items-center gap-2 rounded-full mb-6 text-white px-2 pr-3 py-1 w-min",
        status === "active" ? "bg-green-500" : "bg-gray-900",
        className
      )}
    >
      <StatusIcon status={status} className="text-white" />
      <span className="uppercase text-xs font-medium">{status}</span>
    </div>
  );
};
