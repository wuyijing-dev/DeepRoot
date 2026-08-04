# 0.1.4 SBI 控制台

这一页只讲：**屏幕上的字是怎么从内核流到你的终端上的。**

## 1. 路径

```text
println!
  -> console::_print
  -> sbi::console_putchar / console_write
  -> OpenSBI
  -> UART / QEMU
  -> 你的终端
```

## 2. 两条写输出路径

- `console_putchar`：逐字节写，最保守
- `console_write`：批量写，交互体验更好

## 3. 读输入为什么危险

`console_getchar` 走的是 legacy SBI 0.1 接口，EID 必须是 `0x02`。  
这个地方一旦弄错，shell 往往会表现成：

- 一回车就 unknown
- 收到奇怪的空字节

## 4. 为什么这页属于“启动”

因为你在最早期看到的所有横幅、panic、trap 提示，全靠这条路径。  
如果控制台不可用，内核其实可能已经出问题了，只是你看不见。

下一页：[0.1.5 early trap 与 `stvec`](05-early-trap.md)

