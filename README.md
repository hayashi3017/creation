# creation-backend


## TOC

- [How to Develop](./docs/how-to-develop.md)
- [Environment Variables](./docs/environment-variables.md)


## TODO

- [ ] generate uuid(v7) or ulid by rust not mysql
  - [ ] research mysql column type
  - [ ] reaserch specs between uuid and ulid
  - [ ] research rust ulid crate
    - https://github.com/dylanhart/ulid-rs?tab=readme-ov-file
    - https://zenn.dev/kyoheiu/scraps/b2f26fc1bed3fe
- [ ] use redis crate
- [ ] add refresh token
- [ ] add 2fa
- [ ] add utoipa
- [ ] add testing
  - [Rust sqlxでデータベースに依存した部分のテストを書く](https://zenn.dev/htlsne/articles/rust-sqlx-test)
  - 単体テストならば、[tower::util::Oneshot](https://docs.rs/tower/latest/tower/util/struct.Oneshot.html)が利用できる？
  - [alexliesenfeld/httpmock: HTTP mocking library for Rust.](https://github.com/alexliesenfeld/httpmock)
  - [LukeMathWalker/wiremock-rs: HTTP mocking to test Rust applications.](https://github.com/LukeMathWalker/wiremock-rs)
  - [ ] create architecture of code, module structure
  - [ ] add git hook
- [ ] add mail server or use sendgrid
- [ ] add git tag and git-cliff for versioning
