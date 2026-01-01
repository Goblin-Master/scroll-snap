import { useAppStore } from './store';
import { Overlay } from './components/Overlay';
import { Editor } from './components/Editor';
import { Camera } from 'lucide-react';

function App() {
  const { isCapturing, capturedImage, setIsCapturing } = useAppStore();

  if (isCapturing) {
    return <Overlay />;
  }

  if (capturedImage) {
    return <Editor />;
  }

  return (
    <div className="flex flex-col items-center justify-center h-screen bg-zinc-900 text-white gap-8 select-none">
      <div className="text-center space-y-2">
        <h1 className="text-4xl font-bold tracking-tighter bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
          ScrollSnap
        </h1>
        <p className="text-zinc-400">System-level scrolling screenshots</p>
      </div>
      
      <button 
        onClick={() => setIsCapturing(true)}
        className="group relative flex items-center gap-3 px-8 py-4 bg-indigo-600 hover:bg-indigo-500 rounded-full font-semibold transition-all shadow-lg hover:shadow-indigo-500/25 active:scale-95"
      >
        <Camera className="w-6 h-6" />
        <span>Start Capture</span>
      </button>

      <div className="text-sm text-zinc-500 max-w-xs text-center">
        Click start, drag to select an area.<br/>
        Scroll freely to capture.<br/>
        <span className="font-bold text-indigo-400">Press Esc to stop.</span>
      </div>
    </div>
  );
}

export default App;
