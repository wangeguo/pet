# Pet 桌面宠物系统开发 Roadmap

基于[产品文档](product.md)和[架构文档](architecture.md)
制定。

---

## Phase 1: 基础框架 ✅

**目标**: 搭建项目基础架构和多进程通信机制

- [x] 1.1 项目初始化 - 创建 Cargo workspace
- [x] 1.2 创建 crate 结构 - app, common, tray,
  theater, manager
- [x] 1.3 共享库 `common` - 数据模型、配置读写、存储
- [x] 1.4 配置存储系统 - toml 配置文件读写
- [x] 1.5 主进程框架 - 进程管理器
- [x] 1.6 进程间通信 - 基于配置文件的状态同步
- [x] 1.7 系统托盘进程 - tray-icon 托盘和菜单

---

## Phase 2: 剧场进程（Bevy） ✅

**目标**: 实现宠物 3D 渲染和基础交互

- [x] 2.1 Bevy App 配置
- [x] 2.2 透明无边框窗口
- [x] 2.3 置顶显示
- [x] 2.4 GLB 模型加载
- [x] 2.5 摄像机设置
- [x] 2.6 光照设置
- [x] 2.7 窗口拖拽
- [x] 2.8 点击交互

---

## Phase 3: Replay 系统 ✅

**目标**: 实现行为脚本驱动的宠物动作回放系统

- [x] 3.1 脚本数据结构 - BehaviorScript, Keyframe, Action
- [x] 3.2 脚本加载器 - RON 格式解析
- [x] 3.3 Replay 插件 - 脚本回放状态管理
- [x] 3.4 关键帧系统 - 按时间轴执行动作
- [x] 3.5 动画播放 - PlayAnimation 动作
- [x] 3.6 移动动作 - MoveTo
- [x] 3.7 缩放动作 - Scale
- [x] 3.8 随机选择 - Random
- [x] 3.9 脚本切换 - next 字段自动切换
- [x] 3.10 内置脚本 - idle, walk, happy, sleep
- [x] 3.11 交互触发 - 点击触发脚本切换
- [x] 3.12 弹跳动作 - Bounce
- [x] 3.13 旋转动作 - Spin

---

## Phase 4: IPC 基础设施

**目标**: 建立 UDS 双向实时通信层，为 Brain 做基础

- [ ] 4.1 IPC 消息类型 - 在 common 中定义
  IpcEnvelope、ProcessId、IpcMessage
- [ ] 4.2 UDS 服务端 - app 进程中的 UDS listener
- [ ] 4.3 UDS 客户端 - 子进程连接和收发消息
- [ ] 4.4 消息路由 - app 中的 MessageRouter，
  按 target 转发消息
- [ ] 4.5 Theater IPC 插件 - Bevy 插件，桥接 UDS
  与 ECS 事件
- [ ] 4.6 集成测试 - theater <-> app 消息往返验证

---

## Phase 5: 设置中心

**目标**: 实现集中式配置管理界面（Iced）

- [ ] 5.1 重构 AppConfig - 分组结构
  （GeneralSettings / AppearanceSettings /
  AiSettings / MeshySettings）
- [ ] 5.2 配置迁移 - 旧 flat 结构 -> 新分组结构，
  向后兼容
- [ ] 5.3 创建 settings crate - Iced Application
  基础框架
- [ ] 5.4 Settings: General - 开机自启、语言
- [ ] 5.5 Settings: Appearance - 缩放、位置、置顶、
  透明度
- [ ] 5.6 Settings: AI - provider、API key、model、
  endpoint、personality 配置
- [ ] 5.7 Settings: Meshy AI - API Key
- [ ] 5.8 Settings: About - 版本、构建信息
- [ ] 5.9 auto-start 迁移 - 从 tray 菜单移至
  Settings
- [ ] 5.10 托盘菜单更新 - 新增"设置"入口

---

## Phase 6: 宠物管理器

**目标**: 实现宠物创建和管理界面（技术方案待定）

- [ ] 6.1 确定技术方案 - 评估 Bevy / Web / 其他
- [ ] 6.2 管理器基础框架
- [ ] 6.3 创建宠物界面 - 描述输入 + Meshy AI 集成
- [ ] 6.4 生成进度显示 - Pending/InProgress/Succeeded
- [ ] 6.5 宠物列表 - 展示/切换/删除宠物
- [ ] 6.6 模型预览

---

## Phase 7: AI 中枢 Brain

**目标**: 实现 LLM 驱动的智能行为生成

- [ ] 7.1 brain crate 基础 - 入口、UDS 连接、
  常驻事件循环
- [ ] 7.2 rig.rs 集成 - Agent 构建，支持多 provider
  （OpenAI / Anthropic / Ollama / Custom）
- [ ] 7.3 PetContext - 可用动画列表、交互历史、宠物
  状态管理
- [ ] 7.4 PersonalityManager - 基于配置生成系统提示词
- [ ] 7.5 ScriptGenerator - LLM 输出解析为
  BehaviorScript
- [ ] 7.6 脚本验证 - 确保引用的动画和动作类型合法
- [ ] 7.7 降级策略 - AI 不可用时回退内置脚本
- [ ] 7.8 进程管理 - app ProcessManager 中添加 brain
  （常驻进程，随 tray 一起启动）
- [ ] 7.9 Theater 改造 - 交互事件上报 Brain、
  接收远程脚本执行
- [ ] 7.10 AnimationMap 通告 - Theater 将模型动画列表
  通过 IPC 发送给 Brain
- [ ] 7.11 端到端集成 - 点击 -> brain -> LLM ->
  脚本 -> theater 回放

---

## Phase 8: 体验完善

**目标**: 提升稳定性、完善 AI 交互体验

- [ ] 8.1 首次运行引导 - 自动打开管理器引导创建宠物
- [ ] 8.2 错误处理 - 各模块错误处理和用户提示
- [ ] 8.3 日志系统 - 日志记录和调试支持
- [ ] 8.4 思考中动画 - LLM 等待期间播放过渡动画
- [ ] 8.5 对话记忆 - AI 交互历史的滑动窗口管理
- [ ] 8.6 音效系统 - 行为音效（如鸭子"嘎嘎"叫）
- [ ] 8.7 文字气泡 - Theater 中显示 AI 回复文字
- [ ] 8.8 窗口位置记忆 - 保存并恢复宠物窗口位置
- [ ] 8.9 优雅退出 - 退出时的清理工作
- [ ] 8.10 跨平台适配 - macOS / Windows / Linux
  (X11/Wayland)
- [ ] 8.11 性能优化 - 渲染性能、LLM 调用缓存

---

## Phase 9: 语音交互（Future）

**目标**: 实现语音输入输出

- [ ] 9.1 STT 集成 - whisper.cpp 或云端 API
- [ ] 9.2 TTS 集成 - 文本转语音
- [ ] 9.3 麦克风管理 - 设备选择 + VAD（语音活动检测）
- [ ] 9.4 唤醒词检测
- [ ] 9.5 Voice Settings UI - 语音相关设置界面
- [ ] 9.6 音频输出 - 语音和音效播放

---

## Phase 10: 高级特性（Future）

**目标**: 高级集成和扩展能力

- [ ] 10.1 屏幕感知 - 读取活跃窗口信息，
  上下文感知交互
- [ ] 10.2 MCP 集成 - Brain 作为 MCP host，
  调用外部工具
- [ ] 10.3 外部系统集成 - Brain 作为指挥者对接
  外部 agent 系统（如 OpenClaw），或自建类似的
  多 agent 协作平台

---

## 里程碑

| 里程碑 | 完成标准 |
|--------|---------|
| **M1: 最小可运行** ✅ | Phase 1 + 2，能显示静态 3D 宠物 |
| **M2: 行为系统** ✅ | Phase 3，宠物能自动执行行为脚本 |
| **M3: 实时通信** | Phase 4，进程间可双向实时通信 |
| **M4: 设置系统** | Phase 5，集中式设置可用 |
| **M5: 宠物管理** | Phase 6，能创建/管理宠物 |
| **M6: 智能伴侣** | Phase 7，AI 驱动的动态行为 |
| **M7: 体验就绪** | Phase 8，稳定可用的完整产品 |
| **M8: 语音交互** | Phase 9，语音输入输出 |

---

## 技术栈

| 领域 | 技术方案 |
|------|---------|
| 宠物剧场 | Bevy 0.18 |
| 设置中心 | Iced 0.14 |
| 宠物管理器 | 待定 |
| 系统托盘 | tray-icon 0.21 |
| AI 中枢 | rig.rs |
| 宠物生成 | Meshy AI |
| 实时通信 | Unix Domain Socket |
| 配置存储 | config crate |
| 模型格式 | GLB/glTF |
