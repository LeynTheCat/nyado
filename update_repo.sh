#!/bin/bash
set -e

RELEASE_MODE=false
if [[ "$1" == "--release" || "$1" == "-r" ]]; then
    RELEASE_MODE=true
fi

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Ошибка: не в Git-репозитории"
    exit 1
fi

if [ ! -f Cargo.toml ]; then
    echo "Ошибка: Cargo.toml не найден"
    exit 1
fi

if [ "$RELEASE_MODE" = true ]; then
    CURRENT_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
    if [ -z "$CURRENT_VERSION" ]; then
        echo "Ошибка: не удалось найти версию в Cargo.toml"
        exit 1
    fi

    MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
    MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
    PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3)
    NEW_PATCH=$((PATCH + 1))
    NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"

    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    echo "Версия обновлена: $CURRENT_VERSION -> $NEW_VERSION"
fi

git add .

read -p "Введите сообщение коммита: " COMMIT_MSG
if [ -z "$COMMIT_MSG" ]; then
    echo "Сообщение не может быть пустым"
    exit 1
fi

git commit -m "$COMMIT_MSG"

if [ "$RELEASE_MODE" = true ]; then
    TAG_NAME="v$NEW_VERSION"
    git tag "$TAG_NAME"
    echo "Создан тег: $TAG_NAME"
fi

git push origin HEAD

if [ "$RELEASE_MODE" = true ]; then
    git push origin "$TAG_NAME"
    echo "Тег запушен. Релиз соберётся автоматически."

    echo "Публикация на crates.io..."
    if ! cargo publish --dry-run &>/dev/null; then
        echo "Ошибка: не удалось выполнить cargo publish --dry-run. Проверьте настройки."
        exit 1
    fi
    if ! cargo publish; then
        echo "Ошибка при публикации. Возможно, вы не залогинены. Выполните 'cargo login' и повторите скрипт с --release"
        exit 1
    fi
    echo "Крейт опубликован на https://crates.io/crates/nyado"
else
    echo "Изменения запушены без создания нового релиза."
fi