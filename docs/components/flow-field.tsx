"use client";

import { cn } from "@/lib/utils";
import React, { useRef, useEffect } from "react";
import { createNoise2D } from "simplex-noise";
import { useTheme } from "next-themes";

export const FlowField: React.FC<{ className?: string }> = ({ className }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { resolvedTheme } = useTheme();

  useEffect(() => {
    const canvas = canvasRef.current!;
    const ctx = canvas.getContext("2d")!;
    const noise2D = createNoise2D(); 

    let field: number[][];
    let w: number, h: number;
    const size = 20;
    let columns: number, rows: number, zoom: number;
    let rafId: number = 0;

    function initField() {
      columns = Math.floor(w / size) + 2;
      rows = Math.floor(h / size) + 2;
      field = Array.from({ length: columns }, () => new Array(rows).fill(0));
    }

    function calculateField(time: number) {
      for (let x = 0; x < columns; x++) {
        for (let y = 0; y < rows; y++) {
          const angle = noise2D(x / zoom, y / zoom + time / 5000) * Math.PI * 2;
          field[x][y] = angle;
        }
      }
    }

    function reset() {
      zoom = Math.random() * 90 + 20;
      w = canvas.width = window.innerWidth;
      h = canvas.height = window.innerHeight;
      ctx.strokeStyle = "lightgray";
      initField();
    }

    function clearCanvas() {
      ctx.fillStyle = resolvedTheme === "dark" ? "black" : "white";
      ctx.fillRect(0, 0, w, h);
    }

    function drawField() {
      for (let x = 0; x < columns; x++) {
        for (let y = 0; y < rows; y++) {
          const angle = field[x][y];
          ctx.beginPath();
          const x1 = x * size;
          const y1 = y * size;
          ctx.moveTo(x1, y1);
          ctx.lineTo(
            x1 + Math.cos(angle) * size * 1.5,
            y1 + Math.sin(angle) * size * 1.5
          );
          ctx.stroke();
        }
      }
    }

    function animate(time: number) {
      rafId = requestAnimationFrame(animate);
      calculateField(time);
      clearCanvas();
      drawField();
    }

    reset();
    animate(0);

    window.addEventListener("resize", reset);
    return () => {
      window.removeEventListener("resize", reset);
      if (rafId) cancelAnimationFrame(rafId);
    };
  }, [resolvedTheme]);

  return (
    <div className={cn("relative w-full h-full", className)}>
      <canvas ref={canvasRef} className="w-full h-full block" />
      {/* Gradient overlay */}
      <div className="pointer-events-none absolute inset-0 bg-gradient-to-br dark:from-black dark:via-black/90 from-white dark:to-black/60 via-white/90 to-white/60" />
    </div>
  );
}
