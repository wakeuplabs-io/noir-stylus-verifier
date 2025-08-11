import Link from "next/link";
import { buttonVariants } from "./ui/button";
import { GithubIcon, TwitterIcon } from "lucide-react";

export function Footer() {
  return (
    <footer className="border-t w-full h-16">
      <div className="container flex items-center sm:justify-between justify-center sm:gap-0 gap-4 h-full text-muted-foreground text-sm flex-wrap sm:py-0 py-3 max-sm:px-4">
        <div className="flex items-center gap-3">
          <p className="text-center">
            Built by{" "}
            <Link
              className="px-1 underline underline-offset-2"
              href="https://github.com/wakeuplabs-io"
              target="_blank"
            >
              WakeUp Labs
            </Link>
            for
            <Link
              className="px-1 underline underline-offset-2"
              href="https://arbitrum.io"
              target="_blank"
            >
              Arbitrum
            </Link>
            and{" "}
            <Link
              className="px-1 underline underline-offset-2"
              href="https://noir-lang.org"
              target="_blank"
            >
              Noir
            </Link>
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Link
            href="https://github.com/wakeuplabs-io/noir-stylus-verifier"
            target="_blank"
            className={buttonVariants({
              variant: "ghost",
              size: "icon",
            })}
          >
            <GithubIcon className="h-[1.1rem] w-[1.1rem]" />
          </Link>
          <Link
            href="https://x.com/wakeuplabs"
            target="_blank"
            className={buttonVariants({
              variant: "ghost",
              size: "icon",
            })}
          >
            <TwitterIcon className="h-[1.1rem] w-[1.1rem]" />
          </Link>
        </div>
      </div>
    </footer>
  );
}
