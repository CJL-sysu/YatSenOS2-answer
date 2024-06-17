# YatSenOS2-answer

中山大学 YatSenOS v2 操作系统实验的参考实现仓库，基于 Rust、面向 UEFI 和 x86_64 ，简称YSOS

实验教程仓库 https://github.com/YatSenOS/YatSenOS-Tutorial-Volume-2

实验文档 https://ysos.gzti.me/

参考GGOS: https://github.com/GZTimeWalker/GGOS

## 启动YSOS

```bash
# 使用 make
make run
# 或使用 ysos.py
./ysos.py run
```

进入YSOS后可用 `help` 指令查看所支持的功能

- exit: 退出
- ps: 展示当前所有进程
- app: 展示所有用户程序
- run: 运行用户程序
- clear: 清屏
- help: 打印帮助信息
- cd: 切换文件夹
- cat: 查看文件内容
- ls: 列出文件夹下文件
- 输入程序的路径, 可直接运行程序