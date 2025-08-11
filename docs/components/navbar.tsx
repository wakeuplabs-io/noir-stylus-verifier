import { ModeToggle } from "@/components/theme-toggle";
import { MoveUpRightIcon } from "lucide-react";
import Link from "next/link";
import Anchor from "./anchor";
import { SheetLeftbar } from "./leftbar";
import { page_routes } from "@/lib/routes-config";
import { SheetClose } from "@/components/ui/sheet";

export const NAVLINKS = [
  {
    title: "Documentation",
    href: `/docs${page_routes[0].href}`,
  },
  {
    title: "Examples",
    href: "https://github.com/wakeuplabs-io/noir-stylus-verifier/tree/develop/examples",
    target: "_blank",
  },
];

export function Navbar() {
  return (
    <nav className="w-full border-b h-16 sticky top-0 z-50 bg-background">
      <div className="sm:container mx-auto h-full flex items-center sm:justify-between md:gap-2">
        <div className="flex items-center sm:gap-5 gap-2.5">
          <SheetLeftbar />
          <div className="flex items-center gap-6">
            <div className="lg:flex hidden">
              <Logo />
            </div>
          </div>
        </div>

        <div className="flex items-center sm:justify-normal justify-end sm:gap-3 ml-1 sm:w-fit w-[90%]">
          <div className="flex items-center justify-between sm:gap-2">
            <div className="md:flex hidden items-center  gap-4 text-sm font-medium text-muted-foreground">
              <NavMenu />
            </div>

            <div className="flex ml-4 sm:ml-0">
              <ModeToggle />
            </div>
          </div>
        </div>
      </div>
    </nav>
  );
}

export function Logo() {
  return (
    <Link href="/" className="flex items-center gap-2.5">
      <h2 className="text-md font-bold font-code">Noir Stylus Verifier</h2>
    </Link>
  );
}

export function NavMenu({ isSheet = false }) {
  return (
    <>
      {NAVLINKS.map((item) => {
        const Comp = (
          <Anchor
            key={item.title + item.href}
            activeClassName="!text-primary dark:font-medium font-semibold"
            absolute
            className="flex items-center gap-1 sm:text-sm text-[14.5px] dark:text-stone-300/85 text-stone-800 hover:text-primary dark:hover:text-primary"
            href={item.href}
            target={item.target}
          >
            {item.title} {item.target && <MoveUpRightIcon className="w-4 h-4 font-extrabold" />}
          </Anchor>
        );
        return isSheet ? (
          <SheetClose key={item.title + item.href} asChild>
            {Comp}
          </SheetClose>
        ) : (
          Comp
        );
      })}
    </>
  );
}
