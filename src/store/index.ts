import { create } from 'zustand'

interface AppState {
  isCapturing: boolean
  capturedImage: string | null
  setIsCapturing: (isCapturing: boolean) => void
  setCapturedImage: (image: string | null) => void
}

export const useAppStore = create<AppState>((set) => ({
  isCapturing: false,
  capturedImage: null,
  setIsCapturing: (isCapturing) => set({ isCapturing }),
  setCapturedImage: (capturedImage) => set({ capturedImage }),
}))
