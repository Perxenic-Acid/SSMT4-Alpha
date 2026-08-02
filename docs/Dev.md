# 开发规范

项目前后端分别分为如下几个层级

前端 Vue,TypeScript,Tauri：

后端 Rust:
- commands 负责暴露命令给前端
- services 负责暴露给前端的命令的具体实现
- games 各个游戏提取模型逻辑的具体实现
- common 业务逻辑的抽象复用模型
- helper 具有一定业务逻辑，可能会调用多个utils的工具类
- utils 每个函数都是单一职责的工具类
- config 配置类

# 设计注意事项

- SSMT4中，只有游戏配置名称和游戏预设名称，不存在数据类型文件夹、更新逻辑、3Dmigoto包等可选项了，全部依赖于开发者去添加。
