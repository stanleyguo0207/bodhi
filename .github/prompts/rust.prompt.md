🌐 Copilot 代码生成指令：

1. 所有注释、说明、文档必须使用中文。
2. 命名必须遵循 Rust 官方命名规范：
   - 变量、函数、方法、模块使用蛇形命名（snake_case）
   - 结构体、枚举、特质、类型别名使用帕斯卡命名（PascalCase）
   - 常量、静态变量使用全大写蛇形命名（SCREAMING_SNAKE_CASE）
   - 避免使用关键字作为标识符，冲突时添加下划线后缀（如 type_ 而非 r#type）
3. 注释需符合 RFC1574 规范：
   - 使用 /// 为结构体、函数等项添加文档注释，支持 Markdown 格式
   - 使用 //! 为模块或 crate 添加整体文档注释
   - 文档注释需包含必要的结构化标签（如 # Examples, # Arguments, # Returns, # Errors）
   - 普通注释（//）用于解释复杂逻辑的“为什么”，而非重复代码功能
4. 禁止使用 mod.rs 文件，模块应使用 Rust 2018 风格
   - 子模块放在同名目录下（如 player/skill.rs 对应 player::skill 模块）
5. 游戏服务器特性：
   - 代码需考虑并发安全性（如使用 tokio 异步 runtime、适当的锁机制）
   - 网络模块建议基于 tokio::net 或 tokio-tungstenite 实现
   - 数据持久化可考虑结合 sqlx 或 redis 客户端
   - 错误处理使用 thiserror 定义自定义错误类型，结合 anyhow 处理动态错误
6. 其他要求：
   - 所有注释使用中文
   - 优先使用 Rust 标准库和成熟生态库（如 tokio, serde, log）
   - 避免全局变量，通过依赖注入传递共享状态

请在 Copilot 生成的 Rust 代码中严格遵守上述规范
