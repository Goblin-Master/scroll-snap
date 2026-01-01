import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../store';

export const Overlay = () => {
  const [startPos, setStartPos] = useState<{x: number, y: number, sx: number, sy: number} | null>(null);
  const [selection, setSelection] = useState<{x: number, y: number, w: number, h: number, sx: number, sy: number} | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const { setCapturedImage, setIsCapturing } = useAppStore();

  useEffect(() => {
    const initOverlay = async () => {
      try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        if (!(window as any).__TAURI_INTERNALS__) {
            console.warn("Not running in Tauri environment");
            return;
        }
        const appWindow = getCurrentWindow();
        await appWindow.setFullscreen(true);
        await appWindow.setDecorations(false);
        await appWindow.setAlwaysOnTop(true);
      } catch (e) {
        console.error("Failed to init overlay:", e);
      }
    };
    initOverlay();
  }, []);

  // Listen for capture events
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if (!(window as any).__TAURI_INTERNALS__) return;

    const unlistenComplete = listen<string>('capture-complete', async (event) => {
        console.log("Capture complete");
        setCapturedImage(event.payload);
        setIsCapturing(false);
        await restoreWindow();
    });

    const unlistenError = listen<string>('capture-error', async (event) => {
        console.error("Capture error:", event.payload);
        alert('Capture failed: ' + event.payload);
        setIsCapturing(false);
        await restoreWindow();
    });

    return () => {
        unlistenComplete.then(f => f());
        unlistenError.then(f => f());
    };
  }, [setCapturedImage, setIsCapturing]);

  const restoreWindow = async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if (!(window as any).__TAURI_INTERNALS__) return;
    const appWindow = getCurrentWindow();
    await appWindow.setFullscreen(false);
    await appWindow.setDecorations(true);
    await appWindow.setAlwaysOnTop(false);
    await appWindow.show();
    await appWindow.setFocus();
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    if (isProcessing) return;
    setStartPos({ x: e.clientX, y: e.clientY, sx: e.screenX, sy: e.screenY });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (isProcessing) return;
    if (!startPos) return;
    
    const w = Math.abs(e.clientX - startPos.x);
    const h = Math.abs(e.clientY - startPos.y);
    const x = Math.min(e.clientX, startPos.x);
    const y = Math.min(e.clientY, startPos.y);
    
    const sx = Math.min(e.screenX, startPos.sx);
    const sy = Math.min(e.screenY, startPos.sy);

    setSelection({ x, y, w, h, sx, sy });
  };

  const handleMouseUp = async () => {
    if (isProcessing) return;
    if (selection && selection.h > 10) {
      setIsProcessing(true);
      const appWindow = getCurrentWindow();
      
      // Hide window to allow capturing the content behind it
      // IMPORTANT: We must hide the window so the user can interact with the content below!
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if ((window as any).__TAURI_INTERNALS__) {
          await appWindow.hide();
      }
      
      try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        if (!(window as any).__TAURI_INTERNALS__) {
             alert("请在 Tauri 环境下运行以进行截图");
             setIsProcessing(false);
             return;
        }
        
        const captureRect = {
            x: Math.round(selection.sx),
            y: Math.round(selection.sy),
            width: Math.round(selection.w),
            height: Math.round(selection.h)
        };
        
        // Call backend - this is now non-blocking (async on Rust side spawns a thread)
        // But the invoke itself returns immediately? No, invoke waits for the command to return.
        // So we changed the command to return immediately after spawning a thread.
        await invoke('start_scroll_capture', captureRect);
        
        // We do NOT restore window here. We wait for the event.
        // We do NOT setCapturedImage here. We wait for the event.
        
      } catch (e) {
        console.error(e);
        await restoreWindow();
        alert('启动截图失败: ' + e);
        setIsCapturing(false);
      }
    } else {
        setStartPos(null);
        setSelection(null);
    }
  };

  return (
    <div 
      className={`fixed inset-0 bg-black/30 cursor-crosshair z-50 select-none ${isProcessing ? 'pointer-events-none opacity-0' : ''}`}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      {selection && !isProcessing && (
        <div 
            className="absolute border-2 border-indigo-500 bg-transparent shadow-[0_0_0_9999px_rgba(0,0,0,0.5)]"
            style={{
                left: selection.x,
                top: selection.y,
                width: selection.w,
                height: selection.h
            }}
        />
      )}
      
      {!isProcessing && (
       <div className="absolute top-4 left-1/2 -translate-x-1/2 px-4 py-2 bg-zinc-900 text-white rounded-full text-sm font-medium shadow-lg border border-zinc-700 pointer-events-none">
        拖拽框选区域，松开鼠标开始录制（之后请手动滚动）
      </div>
      )}
    </div>
  );
};
