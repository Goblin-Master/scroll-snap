import { useAppStore } from '../store';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
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

  const handleSave = async () => {
    try {
        // Remove data:image/png;base64, prefix
        const base64Data = capturedImage.split(',')[1];
        
        // Convert base64 to binary
        const binaryString = atob(base64Data);
        const bytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
        }

        const path = await save({
            filters: [{
                name: 'Image',
                extensions: ['png']
            }],
            defaultPath: `scrollsnap-${Date.now()}.png`
        });

        if (path) {
            // Write file using tauri-plugin-fs
            // We need to enable fs plugin in capabilities too? 
            // Actually, `tauri-plugin-dialog` returns the path, but we need `tauri-plugin-fs` to write.
            // Let's use invoke to write? No, plugin-fs is better.
            // But wait, user only installed dialog plugin.
            // Let's add `tauri-plugin-fs` too.
            // Or just use a custom command `save_image(path, base64)`.
            // Custom command is safer if we don't want to expose full FS access.
            // But let's assume we can add FS plugin.
            // Actually, we can use the `save` API? No, `save` just returns a path.
            
            // Let's implement a simple `save_image` command in Rust to avoid setting up FS permissions complexity for now.
            // It's cleaner.
            await invoke('save_image', { path, base64Image: capturedImage });
            alert('Saved successfully!');
        }
    } catch (e) {
        console.error(e);
        alert('Failed to save: ' + e);
    }
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
