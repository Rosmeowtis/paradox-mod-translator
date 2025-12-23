# 术语表

术语表存储路径位于 data/glossary 或 data/glossary_custom。
前者为项目开发者提供的基本术语，随源码库一同更新、维护、发布。
后者为用户自行添加的术语。

本项目采用的术语表结构：

```rust
/// 同一个词汇在不同语言下的表述
pub struct GlossaryItem {
    english: Option<String>, // 英语
    simp_chinese: Option<String>, // 简体中文
    spanish: Option<String>, // 西班牙语
    french: Option<String>, // 法语
    braz_por: Option<String>, // 葡萄牙语
    russian: Option<String>, // 俄语
    german: Option<String>, // 德语
    japanese: Option<String>, // 日语
    korean: Option<String>, // 韩语
    polish: Option<String>, // 波兰语
}
```

为了压缩术语表文件体积，各字段名在序列化/反序列化时按字段顺序数字进行替换。例如：

```json
{
    "1": "hello",
    "2": "你好",
    "3": "hola",
    "4": "bonjour",
}
```

将被读取为

```json
{
    "english": "hello",
    "simp_chinese": "你好",
    "spanish": "hola",
    "french": "bonjour",
}
```

字段的排序以世界语言用量占比决定：

| 排名 | 语言 | 总使用人数（约） |
|------|------|----------------|
| 1 | **英语** (English) | 1.5B |
| 2 | **简体中文** (Chinese) | 1.2B |
| 3 | **西班牙语** (Spanish) | 558.5M |
| 4 | **法语** (French) | 311.9M |
| 5 | **葡萄牙语** (Portuguese) | 266.6M |
| 6  | **俄语** (Russian) | 253.4M |
| 7 | **德语** (German) | 134.0M |
| 8 | **日语** (Japanese) | 125.6M |
| 9 | **韩语** (Korean) | 81.6M |
| 10 | **波兰语** (Polish) | 45.3M |

* 数据来源 https://www.ethnologue.com/insights/ethnologue200/

## 翻译过程中术语表的加载

在翻译过程中，当每翻译一个切片时，搜索源文本，只向大模型提供原文本所包含的术语及其对应目标语言的翻译。