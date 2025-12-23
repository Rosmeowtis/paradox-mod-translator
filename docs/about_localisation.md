# Paradox 本地化语言

本章节说明 Paradox 本地化语言（Paradox Localisation）的基本语法。

Paradox 本地化语言是一种领域特定语言，用于为游戏提供可国际化的、可包含动态内容的富文本。
其文件扩展名是 `.yml`，但实际上并非合法的 YAML 文件，且需要使用 **UTF-8 WITH BOM** 作为文件编码。
文件结构：

- 可选的 **语言标识** 行：如 `l_english:`、`l_simp_chinese:`（可以有多个，以兼容 `localisation/languages.yml`）。
- 多个 **键值对**：`<key>:<number?> "<text>"`，其中 `<number>` 为可选的内部追踪号。
- 注释：以 `#` 开始的单行注释。

文本（`"<text>"`）内可用的标记：

- **颜色**：`§X ... §!`（`X` 为单字符 ID）。
- **参数**：`$name$` 或 `$name|argument$`。`name` 可为本地化键、命令，或脚本变量引用（如 `$@var$` 形式在解析层面等价）。
- **图标**：`£icon|frame£`（`|frame` 可省略），在渲染时嵌入 GFX 图标。
- **命令**：`[text|argument]`，其中 `text` 可参数化；常用于 `Get...`/上下文调用。
- **概念命令（Stellaris）**：`['concept' <rich text>]`，用于链接概念与显示说明文本。
- **文本格式（CK3/Vic3）**：`#format ... #!`，用于样式化文本块；以及 **文本图标**：`@icon!`（以 `@` 开始、以 `!` 结尾）。

示例：

```paradox_localisation
l_english:
 # comment
 key:0 "line\nnext line"
 another_key:0 "§Y$target$§! produces £unity£"
 command_key:0 "Name: [Root.GetName]"
 concept_command_key:0 "['pop_growth', §G+10%§!]"
```

注意事项：

- 冒号后的数字（追踪号）可以省略。
- 文本中的双引号在多数情况下不需要转义，但建议避免不成对的引号。

> [!warning]
> `#format`、`@icon!` 等为特定游戏支持的进阶标记；仅在对应游戏中有效。`['concept' ...]` 仅 Stellaris 支持。

## 翻译过程中的处理

在翻译过程中，同一个文件的 `l_english:` 头键会被省略，并在翻译后处理的合并阶段按对应的目标语言重新加回。
例如从 `english` 到 `simp_chinese`：

```yml
l_english:
  example_a: "Example Apple"
  example_b: "Example Banana"
  example_c: "Example Orange"
```

在预处理切片阶段，`l_english` 会被去除，并自动删除缩进：

```yml
example_a: "Example Apple"
example_b: "Example Banana"
example_c: "Example Orange"
```

随后，接下来的编号阶段，将key替换为按顺序排列的数字：

```yml
1: "Example Apple"
2: "Example Banana"
3: "Example Orange"
```

随后，将上述内容传递给大模型翻译，得到：

```yml
1: "示例苹果"
2: "示例香蕉"
3: "示例橘子"
```

之后进入后处理阶段，将键值重新对应起来：

```yml
example_a: "示例苹果"
example_b: "示例香蕉"
example_c: "示例橘子"
```

如果同一个文件存在多个切片的，将切片组合起来。

最终，将翻译结果重新拼合、添加缩进和翻译目标语言：

```yml
l_simp_chinese:
  example_a: "示例苹果"
  example_b: "示例香蕉"
  example_c: "示例橘子"
```