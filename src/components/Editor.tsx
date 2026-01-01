import { useAppStore } from '../store';
import { invoke } from '@tauri-apps/api/core';
import { Download, Copy, X } from 'lucide-react';

export const Editor = () => {
  const { capturedImage, setCapturedImage } = useAppStore();

  if (!capturedImage) return null;

  const handleCopy = async () => {
    try {
        await invoke('copy_to_clipboard', { base64Image: capturedImage });
        alert('Copied to clipboard!');
    } catch (e) {
        alert('Failed to copy: ' + e);
    }
  };

  const handleSave = () => {
    const link = document.createElement('a');
    link.href = capturedImage;
    link.download = `scrollsnap-${Date.now()}.png`;
    link.click();
  };

  const handleClose = () => {
    setCapturedImage(null);
  };

  return (
    <div className="flex flex-col h-screen bg-zinc-900 text-white">
      <div className="flex items-center justify-between p-4 border-b border-zinc-700 bg-zinc-800">
        <h2 className="text-lg font-semibold">Preview</h2>
        <div className="flex gap-2">
            <button onClick={handleCopy} className="p-2 hover:bg-zinc-700 rounded-md transition-colors" title="Copy to Clipboard">
                <Copy className="w-5 h-5" />
            </button>
            <button onClick={handleSave} className="p-2 hover:bg-zinc-700 rounded-md transition-colors" title="Save">
                <Download className="w-5 h-5" />
            </button>
            <button onClick={handleClose} className="p-2 hover:bg-zinc-700 rounded-md transition-colors" title="Close">
                <X className="w-5 h-5" />
            </button>
        </div>
      </div>
      <div className="flex-1 overflow-auto p-8 flex justify-center items-start bg-zinc-950">
        <img src={capturedImage} alt="Captured" className="max-w-full shadow-2xl rounded-md border border-zinc-800" />
      </div>
    </div>
  );
};
