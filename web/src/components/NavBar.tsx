"use client";

import Link from "next/link";
import { useState } from "react";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
} from "@/components/ui/navigation-menu";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import {
  ChevronRight,
  GitBranch,
  Library,
  BookTemplate,
  Terminal,
  Zap,
  Users,
  Menu,
  X,
} from "lucide-react";

export default function NavBar() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <header className="fixed inset-x-0 top-0 z-50 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b border-border/50">
      <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
        <Link href="/" className="flex items-center gap-2 group">
          <img src="anvil_white.png" alt="Anvil" className="h-6 w-6" />
          <span className="text-base font-semibold tracking-tight">Anvil</span>
        </Link>

        {/* Desktop Navigation */}
        <div className="hidden lg:block absolute left-1/2 transform -translate-x-1/2">
          <NavigationMenu>
            <NavigationMenuList>
              <NavigationMenuItem>
                <NavigationMenuTrigger className="bg-transparent hover:bg-transparent focus:bg-transparent data-[state=open]:text-foreground/90 text-foreground/80">
                  Product
                </NavigationMenuTrigger>
                <NavigationMenuContent className="w-[400px] lg:w-[520px]">
                  <div className="grid grid-cols-2 gap-3 p-3">
                    <Link href="#features" legacyBehavior passHref>
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <BookTemplate className="h-8 w-8" />
                        <div>
                          <div className="font-medium">Universal templates</div>
                          <p className="text-sm text-foreground/70">
                            Generate entire projects across stacks with one
                            prompt.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                    <Link href="#performance" legacyBehavior passHref>
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <Zap className="h-8 w-8" />
                        <div>
                          <div className="font-medium">Rust performance</div>
                          <p className="text-sm text-foreground/70">
                            Blazing fast, reliable, cross-language execution.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                    <Link href="#cli" legacyBehavior passHref>
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <Terminal className="h-8 w-8" />
                        <div>
                          <div className="font-medium">CLI & SDKs</div>
                          <p className="text-sm text-foreground/70">
                            Scriptable workflows for any ecosystem.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                    <Link href="#docs" legacyBehavior passHref>
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <Library className="h-8 w-8" />
                        <div>
                          <div className="font-medium">Docs</div>
                          <p className="text-sm text-foreground/70">
                            Clean and searchable documentation.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                  </div>
                </NavigationMenuContent>
              </NavigationMenuItem>

              <NavigationMenuItem>
                <NavigationMenuTrigger className="bg-transparent hover:bg-transparent focus:bg-transparent data-[state=open]:text-foreground/90 text-foreground/80">
                  Resources
                </NavigationMenuTrigger>
                <NavigationMenuContent className="w-[400px] lg:w-[520px]">
                  <div className="grid grid-cols-2 gap-3 p-3">
                    <Link href="#templates" legacyBehavior passHref>
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <BookTemplate className="h-8 w-8" />
                        <div>
                          <div className="font-medium">Templates Gallery</div>
                          <p className="text-sm text-foreground/70">
                            Browse templates and get inspired.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                    <Link href="#changelog" legacyBehavior passHref>
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <GitBranch className="h-8 w-8" />
                        <div>
                          <div className="font-medium">Changelog</div>
                          <p className="text-sm text-foreground/70">
                            Stay up to date with the latest changes.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                    <Link href="#community" legacyBehavior passHref>
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <Users className="h-8 w-8" />
                        <div>
                          <div className="font-medium">Community</div>
                          <p className="text-sm text-foreground/70">
                            Join the community and get help.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                  </div>
                </NavigationMenuContent>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuTrigger className="bg-transparent hover:bg-transparent focus:bg-transparent data-[state=open]:text-foreground/90 text-foreground/80">
                  Source
                </NavigationMenuTrigger>
                <NavigationMenuContent className="w-[400px] lg:w-[520px]">
                  <div className="grid grid-cols-2 gap-3 p-3">
                    <Link
                      href="https://github.com/amruth-sn/anvil"
                      target="_blank"
                      legacyBehavior
                      passHref
                    >
                      <NavigationMenuLink className="group flex items-start gap-3 rounded-md border border-border/60 p-3 hover:bg-accent/40 transition-colors">
                        <GitBranch className="h-8 w-8" />
                        <div>
                          <div className="font-medium">GitHub</div>
                          <p className="text-sm text-foreground/70">
                            View source code and contribute.
                          </p>
                        </div>
                      </NavigationMenuLink>
                    </Link>
                  </div>
                </NavigationMenuContent>
              </NavigationMenuItem>
            </NavigationMenuList>
          </NavigationMenu>
        </div>

        {/* Desktop Action Buttons */}
        <div className="hidden lg:flex items-center gap-2">
          <Button variant="ghost" asChild>
            <Link href="#docs">Docs</Link>
          </Button>
          <Button className="bg-gradient-to-r from-red-600 via-orange-500 to-amber-400 text-black font-semibold hover:opacity-90">
            Get Started
          </Button>
        </div>

        {/* Mobile Menu */}
        <Sheet open={mobileMenuOpen} onOpenChange={setMobileMenuOpen}>
          <SheetTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className="lg:hidden p-2"
              aria-label="Open mobile menu"
            >
              <Menu className="h-5 w-5" />
            </Button>
          </SheetTrigger>
          <SheetContent
            side="right"
            className="w-[300px] sm:w-[400px] overflow-y-auto"
          >
            <SheetHeader>
              <SheetTitle>Navigation</SheetTitle>
              <SheetDescription className="sr-only">
                Mobile navigation menu
              </SheetDescription>
            </SheetHeader>

            <div className="flex flex-col gap-6 mt-6 pb-8">
              {/* Product Section */}
              <div className="space-y-3">
                <h3 className="font-semibold text-foreground px-2">Product</h3>
                <div className="space-y-2 pl-4">
                  <Link
                    href="#features"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <BookTemplate className="h-5 w-5" />
                    <div>
                      <div className="font-medium">Universal templates</div>
                      <p className="text-sm text-foreground/70">
                        Generate entire projects across stacks
                      </p>
                    </div>
                  </Link>
                  <Link
                    href="#performance"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Zap className="h-5 w-5" />
                    <div>
                      <div className="font-medium">Rust performance</div>
                      <p className="text-sm text-foreground/70">
                        Blazing fast, reliable execution
                      </p>
                    </div>
                  </Link>
                  <Link
                    href="#cli"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Terminal className="h-5 w-5" />
                    <div>
                      <div className="font-medium">CLI & SDKs</div>
                      <p className="text-sm text-foreground/70">
                        Scriptable workflows for any ecosystem
                      </p>
                    </div>
                  </Link>
                  <Link
                    href="#docs"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Library className="h-5 w-5" />
                    <div>
                      <div className="font-medium">Docs</div>
                      <p className="text-sm text-foreground/70">
                        Clean and searchable documentation
                      </p>
                    </div>
                  </Link>
                </div>
              </div>

              {/* Resources Section */}
              <div className="space-y-3">
                <h3 className="font-semibold text-foreground px-2">
                  Resources
                </h3>
                <div className="space-y-2 pl-4">
                  <Link
                    href="#templates"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <BookTemplate className="h-5 w-5" />
                    <div>
                      <div className="font-medium">Templates Gallery</div>
                      <p className="text-sm text-foreground/70">
                        Browse templates and get inspired
                      </p>
                    </div>
                  </Link>
                  <Link
                    href="#changelog"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <GitBranch className="h-5 w-5" />
                    <div>
                      <div className="font-medium">Changelog</div>
                      <p className="text-sm text-foreground/70">
                        Stay up to date with the latest changes
                      </p>
                    </div>
                  </Link>
                  <Link
                    href="#community"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Users className="h-5 w-5" />
                    <div>
                      <div className="font-medium">Community</div>
                      <p className="text-sm text-foreground/70">
                        Join the community and get help
                      </p>
                    </div>
                  </Link>
                </div>
              </div>

              {/* Source Section */}
              <div className="space-y-3">
                <h3 className="font-semibold text-foreground px-2">Source</h3>
                <div className="space-y-2 pl-4">
                  <Link
                    href="https://github.com/amruth-sn/anvil"
                    target="_blank"
                    className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/40 transition-colors"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <GitBranch className="h-5 w-5" />
                    <div>
                      <div className="font-medium">GitHub</div>
                      <p className="text-sm text-foreground/70">
                        View source code and contribute
                      </p>
                    </div>
                  </Link>
                </div>
              </div>

              {/* Action Buttons */}
              <div className="space-y-3 pt-4 border-t border-border/20">
                <Button
                  variant="ghost"
                  asChild
                  className="w-full justify-start"
                ></Button>
                <Button
                  className="w-full bg-gradient-to-r from-red-600 via-orange-500 to-amber-400 text-black font-semibold hover:opacity-90"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  Get Started
                </Button>
              </div>
            </div>
          </SheetContent>
        </Sheet>
      </div>
    </header>
  );
}
