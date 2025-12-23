
# 项目模块结构 (src/)

```
src/
├── main.rs                    # CLI入口点，支持translate/validate/check-api命令
├── lib.rs                     # 库导出和模块声明
├── config/                    # 配置处理
│   ├── mod.rs
│   ├── task.rs               # 翻译任务配置结构
│   ├── client_settings.rs    # 大模型客户端设置
│   └── env.rs                # 环境变量和API密钥管理
├── preprocess/               # 预处理模块
│   ├── mod.rs
│   ├── yaml_fixer.rs         # YAML修复（修复:0格式、引号、缩进）
│   ├── splitter.rs           # 大文件切片
│   └── normalizer.rs         # 文本规范化
├── translate/                # 翻译模块
│   ├── mod.rs
│   ├── api/                  # 大模型API交互
│   │   ├── mod.rs
│   │   ├── client.rs         # HTTP客户端封装
│   │   └── models.rs         # API请求/响应结构
│   ├── glossary.rs           # 术语表加载与管理
│   ├── validator.rs          # 特殊格式验证（£...£ $...$ §...§）
│   └── batcher.rs            # 批处理控制
├── postprocess/              # 后处理模块
│   ├── mod.rs
│   ├── merger.rs             # 合并翻译切片
│   ├── writer.rs             # 写入目标目录
│   └── cleanup.rs            # 清理临时文件
├── utils/                    # 工具函数
│   ├── mod.rs
│   ├── fs.rs                 # 文件系统辅助
│   ├── regex_patterns.rs     # 预编译正则表达式
│   └── token_estimator.rs    # Token估算（用于切片）
└── error.rs                  # 统一错误类型定义
```

# 数据目录结构

```
data/
├── glossary/                 # 默认术语表
│   └── stellaris.json        # Stellaris基础术语（中英对照）
├── glossary_custom/          # 用户自定义术语表
└── prompts/                  # 大模型提示词模板
    └── translate_system.txt  # 翻译系统提示词
```