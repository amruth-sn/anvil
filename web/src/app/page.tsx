"use client";

import NavBar from "@/components/NavBar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Zap, Layers, Rocket, Terminal } from "lucide-react";
import { motion } from "framer-motion";
import { useRef } from "react";

export default function Home() {
  const videoRef = useRef<HTMLVideoElement>(null);

  return (
    <div className="relative min-h-screen bg-background text-foreground">
      <NavBar />

      {/* Hero */}
      <section className="relative isolate pt-24 sm:pt-28 lg:pt-32 overflow-hidden">
        {/* Video background */}
        <div className="absolute inset-0 -z-10">
          <video
            className="h-full w-full object-cover"
            src="sledgehammer.mp4"
            autoPlay
            muted
            loop
            playsInline
            ref={videoRef}
            poster="https://images.unsplash.com/photo-1618181652209-0349b4a6dbad?q=80&w=1600&auto=format&fit=crop"
          />
          <div className="absolute inset-0 bg-black/55" />
          <div className="pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-gradient-to-t from-background" />
        </div>

        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="relative flex flex-col items-center text-center gap-6 py-20 sm:py-28">
            <motion.h1
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.05, duration: 0.6 }}
              className="text-5xl sm:text-6xl lg:text-7xl font-extrabold tracking-tight"
            >
              <span className="bg-gradient-to-r from-red-600 via-orange-500 to-amber-300 bg-clip-text text-transparent">
                Welcome to the Forge.
              </span>
            </motion.h1>

            <motion.p
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1, duration: 0.6 }}
              className="max-w-2xl text-balance text-lg sm:text-xl text-white/85"
            >
              The world's first universal template engine.
            </motion.p>

            <motion.div
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.15, duration: 0.6 }}
              className="flex flex-col sm:flex-row items-center gap-3"
            >
              <Button className="h-11 px-6 text-base font-semibold bg-gradient-to-r from-red-600 via-orange-500 to-amber-400 text-black hover:opacity-90">
                Get Started
              </Button>
              <Button
                variant="outline"
                className="h-11 px-6 text-base border-white/20 bg-black/40 text-white hover:bg-white/10"
              >
                View Docs
              </Button>
            </motion.div>

            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.25, duration: 0.8 }}
              className="mt-8 flex items-center gap-4 text-sm text-white/70"
            >
              <div className="inline-flex items-center gap-2">
                <span className="h-2 w-2 rounded-full bg-red-500" />
                Built with Rust
                <img
                  src="/rustacean.png"
                  alt="Ferris"
                  className="h-4 w-6 ml-1"
                />
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Features */}
      <section id="features" className="relative py-20 sm:py-28">
        <div className="absolute inset-0 -z-10 bg-[radial-gradient(60%_40%_at_50%_0%,rgba(251,146,60,0.15),transparent_60%)]" />
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="mx-auto max-w-3xl text-center mb-12">
            <h2 className="text-3xl sm:text-4xl font-bold tracking-tight">
              Ship projects like you're forging steel
            </h2>
            <p className="mt-3 text-foreground/70">
              Anvil is a modern, platform-agnostic, cross-language template
              engine for scaffolding entire projects across any tech stack.
            </p>
          </div>

          <div className="grid gap-6 sm:gap-8 grid-cols-1 sm:grid-cols-2 lg:grid-cols-4">
            <Card className="border-border/60 bg-background/60 backdrop-blur">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-base font-medium flex items-center gap-2">
                  <Zap className="h-10 w-10 text-orange-400" />
                  Integrate with any third-party service
                </CardTitle>
                <Badge
                  variant="secondary"
                  className="bg-orange-500/15 text-orange-300 border-orange-400/30"
                >
                  New
                </Badge>
              </CardHeader>
              <CardContent className="text-sm text-foreground/70">
                Built in integrations for any AI SDK, pricing engine,
                authentication framework, or database service.
              </CardContent>
            </Card>

            <Card className="border-border/60 bg-background/60 backdrop-blur">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-base font-medium flex items-center gap-2">
                  <Layers className="h-10 w-10 text-red-400" />
                  Use any stack or language
                </CardTitle>
                <Badge
                  variant="secondary"
                  className="bg-red-500/15 text-red-300 border-red-400/30"
                >
                  Universal
                </Badge>
              </CardHeader>
              <CardContent className="text-sm text-foreground/70">
                Use one engine to scaffold React, Rust, Go, Python, and more.
                Mix-and-match stacks with guardrails.{" "}
                <b>Supports Windows, Linux, and macOS.</b>
              </CardContent>
            </Card>

            <Card
              id="performance"
              className="border-border/60 bg-background/60 backdrop-blur"
            >
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-base font-medium flex items-center gap-2">
                  <Rocket className="h-10 w-10 text-amber-400" />
                  Run with peak performance
                </CardTitle>
                <div className="flex items-center gap-1 text-orange-300">
                  <img src="/rustacean.png" alt="Ferris" className="h-8 w-12" />
                </div>
              </CardHeader>
              <CardContent className="text-sm text-foreground/70">
                Built in Rust for speed and reliability. Zero-copy parsing,
                streamed IO, and <i>deterministic execution</i>.
              </CardContent>
            </Card>

            <Card
              id="cli"
              className="border-border/60 bg-background/60 backdrop-blur"
            >
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-base font-medium flex items-center gap-2">
                  <Terminal className="h-10 w-10 text-orange-300" />
                  CLI & SDKs for automation
                </CardTitle>
                <Badge
                  variant="secondary"
                  className="bg-white/5 text-white/70 border-white/10"
                >
                  Dev-first
                </Badge>
              </CardHeader>
              <CardContent className="text-sm text-foreground/70">
                Scriptable CLI and typed SDKs. Automate scaffolds, plug in your
                LLM, and keep outputs reproducible.
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-border/60 py-10">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4">
          <div className="flex items-center gap-2 text-sm text-foreground/70">
            <img src="/anvil_white.png" alt="Anvil" className="h-5 w-5" />
            <span>Anvil.</span>
            <span className="text-foreground/50">
              © {new Date().getFullYear()}
            </span>
          </div>
          <div className="text-sm text-foreground/60">
            Made with <span className="text-orange-400">heat</span>,{" "}
            <span className="text-red-400">steel</span>, and{" "}
            <span className="text-amber-400">love</span>.
          </div>
        </div>
      </footer>
    </div>
  );
}
