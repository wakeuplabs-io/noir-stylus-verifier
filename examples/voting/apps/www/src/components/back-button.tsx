import { MoveLeftIcon } from "lucide-react";
import { Link } from "@tanstack/react-router";

export const BackButton: React.FC<{ to?: string }> = ({ to }) => {
  return (
    <Link
      to={to || ".."}
      className="flex items-center justify-center h-[46px] w-[46px]  border rounded-full"
    >
      <MoveLeftIcon className="w-4 h-4" />
    </Link>
  );
};
