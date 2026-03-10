## Backend for Guest Place

### Для запуска потребуется

- Клонировать репозиторий

```bash
git clone https://github.com/cookies-with-tea/education-platform -b server
```

- Скопировать `.env`

```bash
cp .env.example .env
```

- Заполнить `.env`
- Запустить контейнеры

```bash
docker-compose up -d --build
```

### Для локальной разработки

- Изменить POSTGRES_HOST = `localhost`

```bash
docker-compose up db -d --build
```

<hr />

### Подсказки, чтобы не забыть

**fix postgres with sqlx** - add postgres to features

```bash
cargo add sqlx -F postgres
```

**add new migration**

```bash
cargo install sqlx-cli
sqlx migrate add -r <name>
```

**hot reload**

```bash
cargo install cargo-watch
cargo watch -x run
cargo watch -c -q -x run
```

**additional**

```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
# download visual studio
```
