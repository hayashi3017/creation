# Architecture


This file is inspired by [architecture.md from rust-analyzer](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md).

# Bird's Eye View

wip


# Entry Points

`creation-driver`

This repo's architecture is inspired by below.
- [Rust の新しい HTTP サーバーのクレート Axum をフルに活用してサーバーサイドアプリケーション開発をしてみる - Don't Repeat Yourself](https://blog-dry.com/entry/2021/12/26/002649#%E5%90%84%E3%83%AC%E3%82%A4%E3%83%A4%E3%83%BC%E3%81%AE%E6%8B%85%E5%BD%93%E9%A0%98%E5%9F%9F)
- [Rust の DI を考える –– Part 2: Rust における DI の手法の整理 - paild tech blog](https://techblog.paild.co.jp/entry/2023/06/12/170637)

# Code Map

### `creation-driver`
This depends on usecase.

### `creation-usecase`
This depends on service.
This has responsibility for belows.
- usecase definition
- usecase 


### `creation-service`
This depends on nothing.
DIP (Dependency Inversion Principle) is applied between service and adapter layer. 

### `creation-adapter`
This depends on service's interface.

# Cross-Cutting Concerns

wip
