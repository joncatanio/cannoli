# Source Code

Cannoli is written entirely in the Rust programming language. We provide the lexer, parser, and compiler in separate modules.
However, the parser is dependent upon the lexer, and the compiler is dependent upon the parser.

### Lexer
The lexer fully supports the [Python 3.6.5 grammar](https://docs.python.org/3/reference/grammar.html). It converts a Python
source file into a stream of tokens (Rust iterator) that may be used by a parser.

### Parser
The parser fully supports the [Python 3.6.5 abstract grammar](https://docs.python.org/3/library/ast.html). The token stream,
provided by the lexer, is used to produce an abstract syntax tree (AST). The AST is represented by Rust enums and is defined in
[`ast.rs`](/src/parser/ast.rs).

### Compiler
The compiler supports a subset of Python 3.6.5. It walks the AST and directly outputs Rust code. It does not construct
a control flow graph (CFG).
