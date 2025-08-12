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
            <svg viewBox="0 0 24 24" aria-hidden="true" className="h-[1.1rem] w-[1.1rem] fill-muted-foreground">
              <g>
                <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"></path>
              </g>
            </svg>
          </Link>
        </div>
      </div>
    </footer>
  );
}
