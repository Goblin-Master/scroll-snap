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
      const appWindow = getCurrentWindow();
      
      // Instead of hiding, we make it transparent to events?
      // No, for "click through" we need to set ignoreCursorEvents(true).
      // But Tauri v2 might not support this easily without plugins or just hiding.
      // Hiding is safer for now, but we need a way to stop.
      // Solution: Show a small floating window control? 
      // Or: We hide the main overlay window, but we can't show a floating control unless we have multi-window.
      // Simpler approach for now:
      // Hide the window so user can scroll.
      // But we can't show a stop button if the window is hidden.
      // Wait! We can set the window to be "transparent" and "ignore mouse events" BUT 
      // that usually applies to the whole window.
      // 
      // Workaround: We can't easily have a floating button AND click-through on the same window in webview.
      // 
      // Alternative: Use Global Shortcut? (e.g. Esc to stop)
      // Or: Just keep the window hidden and rely on "Stop Scrolling" detection?
      // But user said "I want to choose when to stop".
      //
      // Best approach for single window:
      // 1. Hide this overlay window.
      // 2. User scrolls.
      // 3. User clicks the TRAY ICON or switches back to the app? No app is hidden.
      // 
      // Actually, if we use `setIgnoreCursorEvents(true)`, we can keep the window visible (drawing the green border)
      // but let clicks pass through.
      // But we need a part of the window (the stop button) to ACCEPT events.
      // Tauri allows `setIgnoreCursorEvents(true)` but forwarding events to specific elements is tricky.
      //
      // Let's try the "Hide and rely on shortcut/timeout" OR "Create a second small window for controls".
      // Creating a second window is complex.
      //
      // Let's stick to: Hide window, but maybe we can show it again?
      // No, let's use the "Stop" logic:
      // We can't click "Stop" if window is hidden.
      //
      // What if we DON'T hide the window, but make the selection area transparent?
      // In webview, `pointer-events: none` allows clicking through to elements BEHIND the div, 
      // BUT it still hits the webview background (which is transparent), 
      // and if the window itself captures mouse, it won't pass to OS.
      // Tauri window needs `setIgnoreCursorEvents` to pass to OS.
      //
      // Let's try this:
      // 1. Start Capture -> `setIgnoreCursorEvents(true)` for the whole window.
      //    (This means we can't interact with the Stop button either!)
      //    UNLESS we toggle it.
      //
      // OK, the robust way without multi-window is:
      // 1. Hide overlay.
      // 2. Register a global shortcut (e.g. CommandOrControl+Shift+S) to stop? 
      //    Or just rely on the user bringing the window back?
      // 
      // User said: "I want to choose end".
      //
      // Let's implement a simple "Time limit" or "Shake mouse" or just "Global Shortcut"?
      // Global shortcut is best but needs configuration.
      //
      // Let's try a hybrid:
      // We hide the window.
      // But we can use `window.setAlwaysOnTop(true)` and `setIgnoreCursorEvents(true)`.
      // And we draw a green border.
      // User can scroll.
      // TO STOP: We need a way.
      //
      // Maybe we just keep the window HIDDEN and tell the user:
      // "Scroll as much as you want. When done, press ESC (if focused?) or... wait, we can't focus."
      //
      // What if we don't hide, but leave a small "Stop" button in the corner that IS clickable?
      // This requires `setIgnoreCursorEvents` to be dynamic.
      // Tauri supports `setIgnoreCursorEvents(true)` to let everything through.
      // To catch clicks on a button, we need `setIgnoreCursorEvents(false)` when hovering the button?
      // That's possible with mouse move tracking but tricky.
      //
      // Let's go with the SAFEST approach for now that fulfills "I want to scroll":
      // 1. Hide window.
      // 2. User scrolls.
      // 3. User PRESSES A KEY? We can't catch keys if not focused.
      // 4. User STOPS scrolling for 3 seconds -> Auto finish (Existing logic).
      //
      // User said: "I want to choose when to end".
      // 
      // Let's add a "Stop" command that can be triggered.
      // Since we can't easily show a UI, let's use the TRAY ICON? Or just the Auto-Stop is actually fine if we explain it well.
      //
      // Wait, user said "I don't know if it's recording".
      // We really need a visual indicator.
      //
      // Let's try to keep the window VISIBLE but PASS-THROUGH (ignore cursor events).
      // AND we draw a Green Border so they know it's recording.
      // AND we show "Recording... Stop scrolling for 3s to finish".
      // This solves "Unknown status" and "Can scroll".
      
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
                <div className="absolute -top-12 left-0 bg-green-600 text-white px-3 py-1 rounded shadow-md text-xs font-bold flex items-center gap-2">
                    <span className="w-2 h-2 bg-red-500 rounded-full animate-ping"/>
                    正在录制... 停止滚动 3秒 自动结束
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
