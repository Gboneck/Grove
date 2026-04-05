import { useRef, useState, useCallback, useEffect } from "react";
import type { Artifact } from "../lib/tauri";
import ArtifactBlock from "./blocks/ArtifactBlock";

interface WorkspaceCanvasProps {
  artifacts: Artifact[];
  onMove: (id: string, x: number, y: number) => void;
  onResize: (id: string, width: number) => void;
  onRemove: (id: string) => void;
  onCollapse: (id: string, collapsed: boolean) => void;
}

interface DragState {
  id: string;
  startX: number;
  startY: number;
  offsetX: number;
  offsetY: number;
}

interface ResizeState {
  id: string;
  startX: number;
  startWidth: number;
}

const GRID_SNAP = 10;
const MIN_WIDTH = 280;
const MAX_WIDTH = 600;

function snap(v: number): number {
  return Math.round(v / GRID_SNAP) * GRID_SNAP;
}

export default function WorkspaceCanvas({
  artifacts,
  onMove,
  onResize,
  onRemove,
  onCollapse,
}: WorkspaceCanvasProps) {
  const canvasRef = useRef<HTMLDivElement>(null);
  const [drag, setDrag] = useState<DragState | null>(null);
  const [resize, setResize] = useState<ResizeState | null>(null);
  const [livePos, setLivePos] = useState<Record<string, { x: number; y: number }>>({});
  const [liveWidth, setLiveWidth] = useState<Record<string, number>>({});

  // Compute canvas size to fit all artifacts + padding
  const canvasWidth = Math.max(
    800,
    ...artifacts.map(a => (livePos[a.id]?.x ?? a.x) + (liveWidth[a.id] ?? a.width) + 40)
  );
  const canvasHeight = Math.max(
    500,
    ...artifacts.map(a => (livePos[a.id]?.y ?? a.y) + 300) // estimate card height
  );

  // Pointer move handler — shared between drag and resize
  const handlePointerMove = useCallback((e: PointerEvent) => {
    if (drag) {
      const x = snap(Math.max(0, e.clientX - drag.offsetX));
      const y = snap(Math.max(0, e.clientY - drag.offsetY));
      setLivePos(prev => ({ ...prev, [drag.id]: { x, y } }));
    }
    if (resize) {
      const delta = e.clientX - resize.startX;
      const w = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, snap(resize.startWidth + delta)));
      setLiveWidth(prev => ({ ...prev, [resize.id]: w }));
    }
  }, [drag, resize]);

  const handlePointerUp = useCallback(() => {
    if (drag) {
      const pos = livePos[drag.id];
      if (pos) onMove(drag.id, pos.x, pos.y);
      setDrag(null);
    }
    if (resize) {
      const w = liveWidth[resize.id];
      if (w) onResize(resize.id, w);
      setResize(null);
    }
  }, [drag, resize, livePos, liveWidth, onMove, onResize]);

  useEffect(() => {
    if (drag || resize) {
      window.addEventListener("pointermove", handlePointerMove);
      window.addEventListener("pointerup", handlePointerUp);
      return () => {
        window.removeEventListener("pointermove", handlePointerMove);
        window.removeEventListener("pointerup", handlePointerUp);
      };
    }
  }, [drag, resize, handlePointerMove, handlePointerUp]);

  const startDrag = useCallback((id: string, e: React.PointerEvent) => {
    const artifact = artifacts.find(a => a.id === id);
    if (!artifact) return;

    const canvasRect = canvasRef.current?.getBoundingClientRect();
    const scrollLeft = canvasRef.current?.parentElement?.scrollLeft ?? 0;
    const scrollTop = canvasRef.current?.parentElement?.scrollTop ?? 0;
    const cx = (canvasRect?.left ?? 0) - scrollLeft;
    const cy = (canvasRect?.top ?? 0) - scrollTop;

    setDrag({
      id,
      startX: artifact.x,
      startY: artifact.y,
      offsetX: e.clientX - (artifact.x + cx),
      offsetY: e.clientY - (artifact.y + cy),
    });
    setLivePos(prev => ({ ...prev, [id]: { x: artifact.x, y: artifact.y } }));
    e.preventDefault();
  }, [artifacts]);

  const startResize = useCallback((id: string, e: React.PointerEvent) => {
    const artifact = artifacts.find(a => a.id === id);
    if (!artifact) return;
    setResize({ id, startX: e.clientX, startWidth: artifact.width || 360 });
    setLiveWidth(prev => ({ ...prev, [id]: artifact.width || 360 }));
    e.preventDefault();
    e.stopPropagation();
  }, [artifacts]);

  return (
    <div
      ref={canvasRef}
      className="relative select-none"
      style={{
        width: canvasWidth,
        minHeight: canvasHeight,
        cursor: drag ? "grabbing" : undefined,
      }}
    >
      {/* Grid dots for spatial feel */}
      <div
        className="absolute inset-0 pointer-events-none opacity-[0.03]"
        style={{
          backgroundImage: "radial-gradient(circle, #d4a853 0.5px, transparent 0.5px)",
          backgroundSize: "20px 20px",
        }}
      />

      {artifacts.map((artifact) => {
        const x = livePos[artifact.id]?.x ?? artifact.x;
        const y = livePos[artifact.id]?.y ?? artifact.y;
        const w = liveWidth[artifact.id] ?? (artifact.width || 360);
        const isDragging = drag?.id === artifact.id;

        return (
          <div
            key={artifact.id}
            className={`absolute transition-shadow duration-200 ${
              isDragging ? "z-50 shadow-2xl shadow-black/40" : "z-10"
            }`}
            style={{
              left: x,
              top: y,
              width: w,
              transition: isDragging ? "none" : "left 0.15s ease, top 0.15s ease, width 0.15s ease",
            }}
          >
            <ArtifactBlock
              id={artifact.id}
              name={artifact.name}
              artifactType={artifact.artifact_type}
              summary={artifact.content.summary || undefined}
              blocks={artifact.content.blocks}
              updatedAt={artifact.updated_at}
              updateCount={artifact.update_count}
              onRemove={onRemove}
              isDragging={isDragging}
              dragHandleProps={{
                draggable: false,
                onDragStart: () => {},
                onDragEnd: () => {},
              }}
              onPointerDownDrag={(e) => startDrag(artifact.id, e)}
            />
            {/* Resize handle */}
            <div
              className="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize opacity-0 hover:opacity-60 transition-opacity"
              onPointerDown={(e) => startResize(artifact.id, e)}
            >
              <svg viewBox="0 0 16 16" className="w-full h-full text-grove-text-secondary">
                <path d="M14 14L8 14M14 14L14 8M14 14L6 6" stroke="currentColor" strokeWidth="1.5" fill="none" />
              </svg>
            </div>
          </div>
        );
      })}
    </div>
  );
}
