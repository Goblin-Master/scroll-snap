# ScrollSnap - 智能滚动截图工具

ScrollSnap 是一款基于 Tauri v2、Rust 和 React 构建的高性能桌面端长截图工具。它能够模拟鼠标滚动事件，自动截取并智能拼接任意应用程序或网页的滚动内容。

## 功能特性

- **跨平台支持**: 支持 Windows、macOS 和 Linux。
- **系统级截图**: 不局限于浏览器，支持任意桌面软件的滚动截图。
- **智能拼接**: 采用像素行哈希匹配算法，精确识别重叠区域，实现无缝拼接。
- **现代化 UI**: 基于 React 和 Tailwind CSS 构建，提供美观的暗色模式界面。
- **高性能**: 核心逻辑由 Rust 驱动，内存占用低，处理速度快。

## 技术栈

- **框架**: Tauri v2
- **前端**: React 18, TypeScript, Tailwind CSS, Zustand, Lucide React
- **后端**: Rust
  - `screenshots`: 跨平台屏幕截图
  - `enigo`: 输入模拟 (鼠标滚动)
  - `image`: 图像处理
  - `arboard`: 系统剪贴板集成

## 安装与使用 (针对普通用户)

如果您只想使用本软件，无需安装开发环境。

1. **下载**: 请前往 Release 页面下载对应系统的安装包或可执行文件 (如 `.exe`, `.dmg`, `.AppImage`)。
2. **运行**: 双击运行程序。
3. **开始截图**:
   - 点击主界面的 **"Start Capture"** (开始截图) 按钮。
   - 屏幕变暗后，**按住鼠标左键并拖拽**，框选您想要截取的滚动区域（例如网页的内容区域）。
   - **松开鼠标**，进入**手动录制模式**。
   - 此时，请**手动慢速滚动**鼠标滚轮，程序会自动检测并拼接新出现的内容。
   - **注意**: 请保持匀速慢滚，避免快速大幅度跳跃，以免拼接失败。
   - **停止**: 当您停止滚动超过 3 秒，或者到达底部无法继续滚动时，程序会自动结束截图并显示结果。
4. **保存结果**:
   - 截图完成后，会显示预览界面。
   - 点击 **Copy** 图标复制到剪贴板。
   - 点击 **Save** (下载) 图标保存为图片文件。

## 开发指南

如果您是开发者，想自己编译或修改代码，请参考以下步骤。

### 前置要求

- **Node.js**: v18 或更高版本
- **Rust**: v1.75 或更高版本
- **系统依赖**:
  - Linux: 需要安装 `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
  - Windows/macOS: 安装标准的构建工具 (如 Visual Studio C++ 负载)。

### 安装步骤

1. **克隆仓库**:
   ```bash
   git clone <repository-url>
   cd scroll-snap
   ```

2. **安装前端依赖**:
   ```bash
   npm install
   ```

3. **开发模式运行**:
   ```bash
   npm run tauri dev
   ```
   这将启动前端热更新服务器和 Tauri 后端。

### 构建发布版 (生成 EXE)

要生成可分发的可执行文件（如 .exe），请运行：

```bash
npm run tauri build
```

构建完成后，安装包和可执行文件将位于：
`src-tauri/target/release/bundle/` 目录下。

- **Windows**: `src-tauri/target/release/bundle/nsis/` (安装包) 或 `msi/`
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Linux**: `src-tauri/target/release/bundle/deb/` 或 `appimage/`

## 常见问题

**Q: 截图拼接错位怎么办？**
A: 可能是因为滚动速度过快或页面含有大量动态元素。您可以尝试重新截图，确保页面已完全加载。

**Q: 为什么我的鼠标自己动了？**
A: 这是软件在模拟滚动操作，属于正常现象。

## 许可证

MIT
