import { GithubIcon } from "lucide-react";

export const Footer = () => {
  return (
    <div className="flex items-center justify-between border-t h-10 px-6">
      <a
        href="https://wakeuplabs.io"
        target="_blank"
        className="text-sm flex items-center gap-2"
      >
        <span>Powered by Wakeup Labs</span>
        <img src="/favicon.svg" alt="Wakeuplabs" className="w-4 h-4" />
      </a>

      <a
        href="https://github.com/wakeuplabs-io/noir-stylus-verifier"
        target="_blank"
        rel="noopener noreferrer"
      >
        <GithubIcon className="w-4 h-4" />
      </a>
    </div>
  );
};
