# 1.15.1 Framebuffer：fbmenu + GUI

对齐：**v1.15.2**（交互修复；功能自 1.15.1）。

在 1.15.0 像素原语之上，**`drivers/fbmenu`**（embed `/fbmenu`）提供简单屏幕菜单与图形终端。设备模块继续放在 **`drivers/`**，不进 `user/`。

## 行为

1. init：`/virtioblk` → `/fbdemo`（画完退出）→ `/fbmenu`
2. fbmenu 配置 ramfb，画菜单（About / Bounce / Terminal）
3. **冒烟**：启动时快速走过 About + Terminal（打串口 marker），然后把菜单交给你
4. **交互**：串口 `w`/`s` 移动，`Enter` 确认；视图内 `q`/`Esc` 返回

注意：在 `deeproot>` 提示符下，**shell 会吃掉按键**；要自己选菜单请：

```bash
./scripts/run-qemu.sh --gui
# 等到 fbmenu: your turn ...
# 或在 shell 里：run fbmenu   （此期间 shell 在 wait，按键给 fbmenu）
```

## 看窗口

```bash
./scripts/run-qemu.sh --gui
```

GTK 显示 ramfb；本终端仍是 UART。需要本机 `DISPLAY`。

## 验收

```bash
git checkout v1.15.2
./scripts/run-qemu.sh --smoke
```

关注：`fbmenu: ramfb ok` → `menu ready` → `select about` → `terminal demo` → `your turn`。

## 下一步

**1.16** virtio-input / seat（真正键盘指针，不再只靠 `SYS_DEBUG_READ`）。
