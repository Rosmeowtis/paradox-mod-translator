# Paradox Mod Translator

本项目是一个基于语言大模型（LLM）的 Paradox 本地化文件翻译工具。

+ 通过 OpenAI 兼容 API 调用 LLM 对文本进行翻译。API Key 需要用户自行配置。
+ 通过 task.toml 文件配置翻译任务，配置项可参考 [task.template.toml](./task.template.toml)
+ 通过术语表对相关游戏术语进行规范化，项目自带术语表可查看 [glossary](./data/glossary/) 文件夹内容，另外，用户还可在数据目录 `/data/glossary_custom` 中以相同的格式添加自定义术语。

## 使用方法

```sh
pmt translate task.toml
```