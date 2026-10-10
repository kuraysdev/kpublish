# Сборка для k3s

Push в master или ручной запуск Release запускает тесты и сборку.
Pull Request только проверяется, без публикации и доступа к App key.
Успешная сборка основной ветки публикует `ghcr.io/kuraysdev/kpublish:COMMIT_SHA`
и открывает PR с immutable digest в `kuraysdev/kurays-infrastructure`.
CI не подключается к серверам или Kubernetes и не выполняет автоматический merge.

## Первоначальная настройка

1. Опубликовать текущие изменения инфраструктуры: workflow, `scripts/update_image.py`
   и `applications/kpublish/values.yaml` должны находиться в `main`.
2. Создать GitHub App с repository permissions: Contents: write,
   Pull requests: write. Установить только на `kurays-infrastructure`.
3. В этом репозитории добавить Actions variable `INFRA_APP_ID` и Actions secret
   `INFRA_APP_PRIVATE_KEY` (PEM key App). Ключ не помещать в Git.
4. Опубликовать изменения этого репозитория в master и проверить Release.
   Для частного GHCR предоставить кластеру отдельный read-only imagePullSecret.
5. Проверить digest PR. Первый переход с локального образа на GHCR и включение
   Argo CD выполняются отдельно под контролем владельца. `enabled: false` CI сохраняет.

`.github/workflows/ci.yaml` — копия общего workflow инфраструктуры: это позволяет
публичному исходному репозиторию работать с приватной инфраструктурой без вызова
приватного reusable workflow. При обновлении общего workflow обновлять эту копию.

Docker context ограничен `.dockerignore`: конфиги, ключи, базы и пользовательские
данные не попадают в образ. Данные и секреты подключаются отдельно при запуске.

Rust multi-stage build выполняет `cargo test --locked` и release build.
В финальном образе только бинарник и templates, пользователь UID/GID 65532.
`/app/config.yaml` подключается из Secret, `/app/public` — из существующего PVC.
Настройка host в Secret должна быть `0.0.0.0:8777`.
Проверка пути в исходниках запрещает выход из public и traversal через symlink.

