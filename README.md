# YatSenOS2-answer

中山大学 YatSenOS v2 操作系统实验的参考实现仓库，基于 Rust、面向 UEFI 和 x86_64 ，简称YSOS

实验教程仓库 https://github.com/YatSenOS/YatSenOS-Tutorial-Volume-2

实验文档 https://ysos.gzti.me/

参考GGOS: https://github.com/GZTimeWalker/GGOS

## 启动YSOS

经过测试，可在 windows, linux 上运行

linux 环境配置参照 https://ysos.gzti.me/wiki/linux/

windows 环境配置参照 https://ysos.gzti.me/wiki/windows/

```bash
# 推荐使用 ysos.py 启动
python ./ysos.py run
# 或使用 make 启动（在有的设备上可能不支持）
make run
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