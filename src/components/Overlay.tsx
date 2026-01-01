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
    // Reset window style but keep it visible
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
      
      try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        if (!(window as any).__TAURI_INTERNALS__) {
             alert("请在 Tauri 环境下运行以进行截图");
             setIsProcessing(false);
             return;
        }
        
        // Calculate physical coordinates
        const dpr = window.devicePixelRatio || 1;
        
        // selection.sx/sy are e.screenX/Y (Logical global coordinates usually)
        // converting them to physical is tricky across browsers/OS
        // But multiplying by DPR is the standard way to get physical pixels from logical CSS pixels
        
        const captureRect = {
            x: Math.round(selection.sx * dpr),
            y: Math.round(selection.sy * dpr),
            width: Math.round(selection.w * dpr),
            height: Math.round(selection.h * dpr)
        };
        
        console.log(`Capture Rect (Physical): ${JSON.stringify(captureRect)}, DPR: ${dpr}`);
        
        // Call backend
        // Backend will handle hiding the window to ensure it's synced with capture start
        await invoke('start_scroll_capture', captureRect);
        
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
      className={`fixed inset-0 bg-black/30 cursor-crosshair z-50 select-none ${isProcessing ? 'bg-transparent' : ''}`}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      {selection && (
        <div 
            className={`absolute border-2 ${isProcessing ? 'border-green-500 border-4 animate-pulse' : 'border-indigo-500'} bg-transparent shadow-[0_0_0_9999px_rgba(0,0,0,0.5)]`}
            style={{
                left: selection.x,
                top: selection.y,
                width: selection.w,
                height: selection.h,
                // When processing, we want the shadow to be transparent so we can see?
                // Actually if we setIgnoreCursorEvents(true), the whole window is transparent to clicks.
                // Visually, we want to remove the dark overlay when recording so user sees content clearly.
                boxShadow: isProcessing ? 'none' : '0 0 0 9999px rgba(0,0,0,0.5)'
            }}
        >
            {isProcessing && (
                <div className="fixed top-4 left-1/2 transform -translate-x-1/2 bg-zinc-900/90 text-white px-4 py-2 rounded-lg shadow-xl border border-zinc-700 text-sm font-medium flex items-center gap-3 z-50">
                    <span className="relative flex h-3 w-3">
                      <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                      <span className="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
                    </span>
                    正在录制... 按 Esc 停止
                </div>
            )}
        </div>
      )}
      
      {!isProcessing && (
       <div className="absolute top-4 left-1/2 -translate-x-1/2 px-4 py-2 bg-zinc-900 text-white rounded-full text-sm font-medium shadow-lg border border-zinc-700 pointer-events-none">
        拖拽框选区域，松开鼠标开始录制
      </div>
      )}
    </div>
  );
};
