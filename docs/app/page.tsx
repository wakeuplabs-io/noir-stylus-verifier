import { buttonVariants } from "@/components/ui/button";
import { page_routes } from "@/lib/routes-config";
import Link from "next/link";
import { FlowField } from "@/components/flow-field";

export default function Home() {
  return (
    <div className="flex flex-1 flex-col justify-center sm:py-8 py-14 pb-8">
      <h1 className="text-[1.80rem] leading-8 md:leading-[4.5rem] font-bold mb-4 sm:text-6xl text-left">
        <span className="font-black">Generate and deploy</span> <br /> Stylus
        verifiers for your <br /> Noir ZK apps in minutes_
      </h1>
      <p className="mb-8 md:text-lg text-base  max-w-[1200px] text-muted-foreground text-left">
        Unlock the full potential of Stylus by deploying your Noir zero-
        <br />
        knowledge with optimal gas efficiency and seamless <br /> integration.
        Build privacy-first applications you can trust.
      </p>
      <div className="sm:flex sm:flex-row grid grid-cols-2 items-center sm;gap-5 gap-3 mb-8">
        <Link
          href={`/docs${page_routes[0].href}`}
          className={buttonVariants({ className: "px-6", size: "lg" })}
        >
          Quick Start
        </Link>
        <Link
          href="https://github.com/wakeuplabs-io/noir-stylus-verifier"
          target="_blank"
          className={buttonVariants({
            variant: "secondary",
            className: "px-6",
            size: "lg",
          })}
        >
          View on GitHub
        </Link>
      </div>

      <FlowField className="absolute top-0 left-0 w-full h-full -z-10" />
    </div>
  );
}
