# 1.15.1 Framebuffer：fbmenu + GUI

对齐：**v1.15.1**。

在 1.15.0 像素原语之上，**`drivers/fbmenu`**（embed `/fbmenu`）提供简单屏幕菜单与图形终端。设备模块继续放在 **`drivers/`**，不进 `user/`。

## 行为

1. init：`/virtioblk` → `/fbdemo`（画完退出）→ `/fbmenu`
2. fbmenu 重新配置 ramfb，画菜单（About / Bounce / Terminal）
3. **冒烟**：定时器自动选 About、再跑 Terminal demo（不抢串口）
4. **交互**（`--gui`）：串口 `w`/`s` 移动，`Enter` 确认；视图内 `q`/`Esc` 返回

## 看窗口

```bash
./scripts/run-qemu.sh --gui
```

GTK 显示 ramfb；本终端仍是 UART。需要本机 `DISPLAY`。

## 验收

```bash
git checkout v1.15.1
./scripts/run-qemu.sh --smoke
```

关注：`fbmenu: ramfb ok` → `menu ready` → `select about` → `terminal demo`，以及 `init: fbmenu loaded`。

## 下一步

**1.16** virtio-input / seat（真正键盘指针，不再只靠 `SYS_DEBUG_READ`）。
