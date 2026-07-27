# Стадія 2: перетворити готовий застосунок на продукт, який роздається й оновлюється

Цей документ — робоче завдання для агента в **новій сесії, без жодного контексту**. Він зібраний
з фактичного впровадження в проєкті `SpiritUrban/git-manager` (Tauri v2 + React у pnpm-монорепо),
де кожне правило нижче оплачене реальною поламкою.

**Стадія 1** — застосунок написаний і працює локально.
**Стадія 2** — те, що описано тут: CI, релізи під усі платформи, сайт-вітрина з завантаженнями,
автооновлення, і присутність автора в продукті.

---

## 0. Як агенту користуватися цим документом

1. **Прочитати повністю до першої дії.** Половина правил тут — про те, чого не видно, поки не
   зламається.
2. **Не копіювати наосліп.** Розділ 6 містить робочі файли з плейсхолдерами у кутових дужках —
   їх треба замінити значеннями з профілю проєкту (розділ 2).
3. **Не питати того, що вже вирішено тут.** Рішення в розділах 3, 5 і 7 — прийняті, обґрунтовані
   й перевірені. Питати варто лише про те, що в розділі 2 позначено як «з'ясувати».
4. **Не вважати, що чуже середовище схоже на це.** Спершу з'ясувати стек і структуру репозиторію,
   потім планувати.
5. **Кожне твердження про стан CI перевіряти фактом, а не припущенням.** Розділ 9 дає команди.

---

## 1. Цільовий стан

Після впровадження мусить працювати таке:

| Що | Як перевіряється |
|---|---|
| CI на кожен пуш: лінт, типи, тести | зелений ран у Actions |
| Реліз на пуш тега `v*.*.*`: збірка під усі платформи | GitHub Release з інсталяторами |
| Сайт на GitHub Pages з кнопками завантаження | посилання віддають файл (HTTP 206) |
| Сайт оновлюється сам після релізу | версія в маніфесті = версія релізу |
| Автооновлення в застосунку | встановлена копія показує банер і оновлюється |
| Присутність автора | у застосунку, на сайті, у README, у метаданих бінарника |

---

## 2. Профіль проєкту — з'ясувати до початку

Агент має отримати відповіді (спитати користувача або визначити з репозиторію):

| Питання | Навіщо |
|---|---|
| Стек: Tauri+React / Python+React / інше | визначає, чи є розділ 6.2 і автооновлення взагалі |
| Це десктопний бінарник чи вебзастосунок? | вебзастосунок не має інсталяторів і апдейтера |
| Монорепо чи один пакет? Пакетний менеджер? | від цього залежать шляхи й команди у воркфлоу |
| `productName`, ідентифікатор, `owner/repo` | імена артефактів, URL сайту, ендпоінт апдейтера |
| Чи вже є теги/релізи в репозиторії? | не можна переставляти опублікований тег |
| Гілка за замовчуванням | тригери воркфлоу |

**Плейсхолдери, що вживаються далі:**
`<OWNER>`, `<REPO>`, `<PRODUCT_NAME>`, `<DESKTOP_DIR>` (напр. `apps/desktop`),
`<DESKTOP_PKG>` (напр. `@scope/desktop`), `<SITE_DIR>`, `<PM_VERSION>`.

---

## 3. Залізні правила

Кожне з них — наслідок реальної поламки. Порушення будь-якого коштує від години до дня.

### Збірка та CI

1. **`Cargo.lock` мусить бути в git.** Це застосунок, не бібліотека. Tauri CLI читає лок, щоб
   звірити версії Rust-крейтів з npm-пакетами. Без лока він порівнює npm-версії з **рядками-вимогами**
   з `Cargo.toml`, які завжди відстають, і падає з `Found version mismatched Tauri packages`.
   Локально це не відтворюється ніколи — лок там є завжди.
2. **Rust-джоба CI мусить зібрати фронтенд перед `cargo`-командами.** `tauri::generate_context!`
   вбудовує бандл на етапі компіляції, а `dist/` у `.gitignore`. Без цього `cargo clippy` падає з
   `error: proc macro panicked` і кодом 101, що виглядає як проблема Rust-коду.
3. **Ніякого `macos-13` у матриці.** Ярлик не отримує раннера: джоба висить у Queued годинами й
   не дає рану завершитися. Intel-збірка кросс-компілюється з `macos-latest` через
   `--target x86_64-apple-darwin`.
4. **Ключі матриці — без дефісів.** `${{ matrix.rust-targets }}` парситься як віднімання.
   Тільки підкреслення: `rust_targets`.
5. **Дві джоби на одному раннері мусять мати різні ключі кешу**, інакше затирають кеш одна одної.
6. **Код під `#[cfg(target_os = "...")]` компілюється лише на своїй платформі.** Параметр,
   використаний тільки у Windows-гілці, дає `unused_variables` на Linux, а з `-D warnings` це
   помилка. З Windows такого не побачити. Прийом для перевірки: мінімальний крейт без залежностей
   і `cargo clippy --target x86_64-unknown-linux-gnu`.
7. **Перед першим тегом прогнати `cargo fmt` один раз.** Інакше `--check` червоний на десятках
   файлів.

### Реліз

8. **`tauriScript` обов'язковий у pnpm-монорепо.** `tauri-action` визначає пакетний менеджер за
   лок-файлом у `projectPath`, а в монорепо лок лежить у корені — і дія відкочується на npm.
   Форма з фільтром не залежить від робочої теки:
   `tauriScript: 'pnpm --filter <DESKTOP_PKG> tauri'`.
9. **У десктоп-пакеті мусить бути скрипт `"tauri": "tauri"`.** Без нього `<pm> run tauri` падає з
   `Missing script`.
10. **`tagName` треба захистити умовою.** Без неї ручний запуск створює реліз з тегом `main`.
    `tagName: ${{ startsWith(github.ref, 'refs/tags/') && github.ref_name || '' }}`
11. **Секрети підпису — на рівні джоби, не кроку.** Якщо їх задати лише на кроці `tauri-action`,
    будь-який інший крок, що робить збірку, впаде з
    `A public key has been found, but no private key`.
12. **`includeUpdaterJson: true`** — без `latest.json` ендпоінт апдейтера віддає 404 і жоден
    клієнт ніколи не побачить оновлення.
13. **Опублікований тег не переставляти.** Дозволено лише поки реліз з нього не створився.

### Сайт

14. **`release: types: [published]` не працює як тригер.** Реліз створює `GITHUB_TOKEN`, а GitHub
    навмисно не запускає воркфлоу від подій цього токена. Сайт деплоїться **залежною джобою
    всередині реліз-рану** через `workflow_call`.
15. **Ніколи не хардкодити імена артефактів.** Tauri іменує бандли за `productName`, а GitHub
    замінює пробіли на крапки: `<PRODUCT_NAME>` = `Git Manager` → `Git.Manager_0.1.2_x64-setup.exe`.
    Брати імена з GitHub API.
16. **Платформу визначати за розширенням, а не за словом у назві.** `.rpm` і `.app.tar.gz` не
    містять жодного платформного слова й інакше потрапляють у Windows.
17. **Фільтрувати `.sig` і `latest.json`** зі списку завантажень — це не збірки.
18. **Ніколи не хардкодити версію** ні в UI, ні у фолбеку маніфесту, ні в README. Читати з
    бандла (`getVersion()` у Tauri) або з `package.json`.
19. **Сайт на Pages живе в підкаталозі** `https://<OWNER>.github.io/<REPO>/`. Звідси:
    `base` у vite, `%BASE_URL%` в `index.html`, `import.meta.env.BASE_URL` у рантайм-запитах.

### Діагностика

20. **Логи ранів недоступні без авторизації навіть у публічному репозиторії, а анотації —
    доступні.** Ставити перехоплення виводу в анотації **одразу**, а не після третього невдалого
    рану. Шаблон у розділі 6.4.
21. **Не фільтрувати вивід grep'ом за очікуваним форматом** — виводити хвіст логу цілком.
    Перша версія такого кроку промовчала саме тому, що шукала `файл:рядок:колонка`, а помилка
    була без локації.
22. **Тривалість падіння каже, де шукати:** 1–3 с — конфігурація; 20–40 с — встановлення пакетів
    чи дрібні кроки; хвилини — справді код.
23. **«Локально працює, в CI ні» — це різниця середовищ.** Шукати в такому порядку: чого немає в
    git (`git check-ignore -v`), що є на CI (порожні секрети приходять порожніми рядками), інша ОС.

### Іконки

24. **Перевіряти вміст іконок, а не наявність файлів.** У згаданому проєкті всі шість іконок були
    заглушками 1×1 піксель по 70 байт, причому `icon.icns` був звичайним PNG із чужим розширенням.
    Збірка про це не повідомляє — видно лише на встановленому застосунку.
    Генерувати весь набір з одного джерела: `tauri icon app-icon.svg`.
25. **У SVG-джерелі не писати `--` у комментарях** — XML це забороняє, `tauri icon` падає з
    `ParsingFailed(InvalidComment)`.
26. **Іконку інсталятора задавати окремо:** `bundle.windows.nsis.installerIcon`. За замовчуванням
    інсталятор має стандартний значок NSIS. MSI власну іконку отримати не може в принципі.
27. **Локальна перезбірка може лишити стару іконку.** `embed-resource` перекомпілює ресурс лише
    коли змінюється **текст** `resource.rc`; зміна вмісту `icon.ico` не відстежується, і навіть
    `cargo clean -p` цей артефакт не чіпає. Лікується видаленням
    `target/release/build/<crate>-*/out/resource.{rc,lib}`. На CI не виникає — target холодний.

---

## 4. Порядок робіт

Саме в цій послідовності. Пропуск фази A гарантує години розбору незрозумілих падінь.

**Фаза A — ручні кроки на GitHub** (розділ 5). Без них решта не запрацює.

**Фаза B — підготовка репозиторію:**
- прибрати `Cargo.lock` з `.gitignore`, закомітити його;
- додати скрипт `"tauri": "tauri"` у десктоп-пакет;
- створити `scripts/sync-version.mjs` і `scripts/check-version.mjs` (розділ 6.5);
- згенерувати іконки з одного SVG-джерела.

**Фаза C — воркфлоу** (розділ 6.1–6.3).

**Фаза D — сайт** (розділ 6.6): `base`, `%BASE_URL%`, маніфест завантажень.

**Фаза E — присутність автора** (розділ 7).

**Фаза F — перевірка перед першим тегом:**
1. локально: `cargo fmt --check`, `cargo clippy -- -D warnings`, тести, повна збірка;
2. пуш у `main` → CI і Pages зелені;
3. **ручний запуск релізу** (`Actions → Release → Run workflow`) — збере всі платформи, але
   нічого не опублікує. Найдешевший спосіб зловити 90% проблем;
4. тільки після цього тег.

---

## 5. ⚠️ Ручні кроки на GitHub — агент їх виконати не може

**Це має зробити власник репозиторію. Поки не зроблено — пайплайн буде падати незрозуміло.**

| # | Що | Де | Симптом, якщо пропустити |
|---|---|---|---|
| 1 | Активний платіжний метод, ненульовий spending limit | Settings → Billing and plans | Джоба падає за 3 с з порожнім списком кроків: «The job was not started because recent account payments have failed». Звичайний CI при цьому може працювати — блокуються джоби з `environment` |
| 2 | Pages: Source = **GitHub Actions** | Settings → Pages | Сайт 404, `/repos/.../pages` теж 404 |
| 3 | Оточення `github-pages`: дозволити гілку `main` **і тег** `v*.*.*` | Settings → Environments → github-pages → Deployment branches and tags | `Tag "v0.1.0" is not allowed to deploy to github-pages due to environment protection rules` |
| 4 | Секрети `TAURI_SIGNING_PRIVATE_KEY` і `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Settings → Secrets and variables → Actions | Збірка падає: `public key found, but no private key` |
| 5 | Права воркфлоу на запис (або `permissions: contents: write` у джобі) | Settings → Actions → General | Реліз не створюється |

**Генерація ключів апдейтера** (інтерактивна, запитує пароль — агент не може):

```bash
pnpm --filter <DESKTOP_PKG> tauri signer generate -w .tauri-key
```

Далі: приватний ключ → секрет `TAURI_SIGNING_PRIVATE_KEY`; пароль → секрет
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; **публічний** ключ (`.tauri-key.pub`) → в
`tauri.conf.json` → `plugins.updater.pubkey`. Додати `.tauri-key*` у `.gitignore`.

> **Порядок критичний.** Не вмикати `bundle.createUpdaterArtifacts` до того, як секрети додані:
> інакше **всі** збірки релізу стануть червоними.

> **Втрата ключа або пароля незворотна.** Без них підписати оновлення неможливо, а зміна ключа
> означає, що всі встановлені копії перестануть приймати оновлення.

PAT не потрібен. Вбудованого `GITHUB_TOKEN` достатньо — з єдиним винятком у правилі 14.

---

## 6. Робочі файли

Перевірені в бою. Замінити плейсхолдери.

### 6.1 `.github/workflows/ci.yml` — ключові моменти

```yaml
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: <PM_VERSION> }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: 'pnpm' }
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: <DESKTOP_DIR>/src-tauri }

      - name: Install Linux GUI Dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
            libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

      # Правило 2: без цього кроку cargo падає з "proc macro panicked"
      - name: Build Desktop Frontend
        run: |
          pnpm install --frozen-lockfile
          pnpm build:desktop

      - name: Check Rust Formatting
        run: cargo fmt --manifest-path <DESKTOP_DIR>/src-tauri/Cargo.toml --check

      - name: Clippy Lints
        run: |
          set -o pipefail
          cargo clippy --manifest-path <DESKTOP_DIR>/src-tauri/Cargo.toml \
            --message-format=short -- -D warnings 2>&1 | tee clippy.log

      - name: Surface Clippy diagnostics
        if: failure()
        run: |
          if [ ! -s clippy.log ]; then
            echo "::error title=Clippy::clippy.log is missing or empty"
            exit 0
          fi
          log=$(tail -c 6000 clippy.log)
          log="${log//'%'/'%25'}"
          log="${log//$'\r'/'%0D'}"
          log="${log//$'\n'/'%0A'}"
          echo "::error title=Clippy output::$log"

      - name: Run Rust Unit Tests
        run: cargo test --manifest-path <DESKTOP_DIR>/src-tauri/Cargo.toml
```

### 6.2 `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags: ['v*.*.*']
  workflow_dispatch:

jobs:
  validate-version:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: <PM_VERSION> }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: 'pnpm' }
      - run: pnpm install --frozen-lockfile
      - run: pnpm version:check

  build-tauri:
    needs: validate-version
    name: Build Desktop App (${{ matrix.label }})
    permissions:
      contents: write
    # Правило 11: на рівні джоби, не кроку
    env:
      TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
      TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
    strategy:
      fail-fast: false
      matrix:
        # Правила 3 і 4: без macos-13, ключі без дефісів
        include:
          - { platform: 'windows-latest', label: 'windows-x64', args: '', rust_targets: '' }
          - { platform: 'ubuntu-22.04',   label: 'linux-x64',   args: '', rust_targets: '' }
          - { platform: 'macos-latest',   label: 'macos-arm64',
              args: '--target aarch64-apple-darwin', rust_targets: 'aarch64-apple-darwin' }
          - { platform: 'macos-latest',   label: 'macos-x64',
              args: '--target x86_64-apple-darwin',  rust_targets: 'x86_64-apple-darwin' }
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: <PM_VERSION> }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: 'pnpm' }
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: '${{ matrix.rust_targets }}' }
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: <DESKTOP_DIR>/src-tauri
          key: ${{ matrix.label }}          # правило 5

      - name: Install Linux GUI Dependencies
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
            libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

      - run: pnpm install --frozen-lockfile

      # Правило 20: tauri-action показує лише код виходу. Той самий білд
      # спочатку своїм кроком, щоб побачити помилку.
      - name: Build Desktop App
        shell: bash
        run: |
          set -o pipefail
          pnpm --filter <DESKTOP_PKG> tauri build ${{ matrix.args }} 2>&1 | tee build.log

      - name: Surface build output
        if: failure()
        shell: bash
        run: |
          if [ ! -s build.log ]; then
            echo "::error title=Build::build.log is missing or empty"
            exit 0
          fi
          log=$(tail -c 6000 build.log)
          log="${log//'%'/'%25'}"
          log="${log//$'\r'/'%0D'}"
          log="${log//$'\n'/'%0A'}"
          echo "::error title=Build output::$log"

      - name: Build Tauri Application
        uses: tauri-apps/tauri-action@v0.5
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          projectPath: './<DESKTOP_DIR>'
          tauriScript: 'pnpm --filter <DESKTOP_PKG> tauri'   # правило 8
          tagName: ${{ startsWith(github.ref, 'refs/tags/') && github.ref_name || '' }}
          releaseName: ${{ startsWith(github.ref, 'refs/tags/') && format('<PRODUCT_NAME> {0}', github.ref_name) || '' }}
          releaseBody: 'See release notes for <PRODUCT_NAME> ${{ github.ref_name }}.'
          releaseDraft: false
          prerelease: false
          includeUpdaterJson: true                            # правило 12
          args: ${{ matrix.args }}

  # Правило 14
  deploy-site:
    needs: build-tauri
    if: startsWith(github.ref, 'refs/tags/')
    permissions:
      contents: read
      pages: write
      id-token: write
    uses: ./.github/workflows/pages.yml
```

### 6.3 `.github/workflows/pages.yml`

```yaml
name: Deploy GitHub Pages

on:
  push:
    branches: [main]
    paths:
      - '<SITE_DIR>/**'
      - 'packages/**'
      - 'scripts/generate-download-manifest.mjs'
  workflow_call:        # правило 14
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: 'pages'
  cancel-in-progress: true

jobs:
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: <PM_VERSION> }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: 'pnpm' }
      - run: pnpm install --frozen-lockfile

      - name: Generate Download Manifest
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}   # без токена 60 запитів/год на IP, далі 403
        run: node scripts/generate-download-manifest.mjs

      - name: Report resolved release
        run: |
          v=$(node -p "require('./<SITE_DIR>/public/download-manifest.json').version")
          n=$(node -p "require('./<SITE_DIR>/public/download-manifest.json').assets.length")
          echo "::notice title=Download manifest::ref=${GITHUB_REF_NAME} -> version ${v}, ${n} assets"

      - name: Build Marketing Site
        env:
          GITHUB_PAGES: 'true'
        run: pnpm build:site

      - uses: actions/configure-pages@v4
      - uses: actions/upload-pages-artifact@v3
        with: { path: './<SITE_DIR>/dist' }
      - id: deployment
        uses: actions/deploy-pages@v4
```

### 6.4 Шаблон «показати помилку в анотації»

Додавати до будь-якого кроку, який може впасти незрозуміло. Екранування `%`, CR і LF обов'язкове.

```yaml
      - name: <Step>
        shell: bash
        run: |
          set -o pipefail
          <команда> 2>&1 | tee step.log

      - name: Surface output
        if: failure()
        shell: bash
        run: |
          if [ ! -s step.log ]; then
            echo "::error title=Step::step.log is missing or empty"
            exit 0
          fi
          log=$(tail -c 6000 step.log)
          log="${log//'%'/'%25'}"
          log="${log//$'\r'/'%0D'}"
          log="${log//$'\n'/'%0A'}"
          echo "::error title=Step output::$log"
```

### 6.5 Синхронізація версій

Версія дублюється в багатьох файлах, і джоба `validate-version` звіряє їх **між собою і з іменем
тега**. `sync-version.mjs <version>` мусить оновити:
всі `package.json`, `tauri.conf.json`, `Cargo.toml` **і `Cargo.lock`** (лише рядок версії свого
пакета — регексом, без запуску cargo, щоб не тягнути мережу).

`check-version.mjs` звіряє все це плюс `GITHUB_REF_NAME`, якщо він починається з `v`.

### 6.6 Маніфест завантажень

Ключова логіка (правила 15–18):

```js
const ref = process.env.GITHUB_REF_NAME || '';
const isTag = /^v\d+\.\d+\.\d+$/.test(ref);
const apiUrl = isTag
  ? `https://api.github.com/repos/${owner}/${repo}/releases/tags/${ref}`
  : `https://api.github.com/repos/${owner}/${repo}/releases/latest`;

const headers = { 'User-Agent': '<REPO>-site-builder' };
if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;

// .sig і latest.json — не збірки
const assets = (data.assets || []).filter((a) => {
  const n = a.name.toLowerCase();
  return !n.endsWith('.sig') && n !== 'latest.json';
}).map((a) => {
  const n = a.name.toLowerCase();
  let platform = 'windows';
  if (n.includes('macos') || n.includes('darwin') || n.endsWith('.dmg') || n.endsWith('.app.tar.gz')) platform = 'macos';
  else if (n.includes('linux') || n.endsWith('.appimage') || n.endsWith('.deb') || n.endsWith('.rpm')) platform = 'linux';
  const architecture = n.includes('arm64') || n.includes('aarch64') ? 'arm64' : 'x64';
  return { platform, architecture, fileName: a.name, downloadUrl: a.browser_download_url };
});
```

**Фолбек, коли релізу ще немає:** `assets: []` і версія з `package.json`. Ніколи не вигадувати
імена файлів — кнопка має вести на сторінку релізів, а не на 404.

**На сайті кнопку зіставляти з ассетом за платформою + архітектурою + суфіксом файлу.** Лише
платформи й архітектури замало: під Windows два пакети (`.exe` і `.msi`), під Linux теж
(`.AppImage` і `.deb`) — і картка «MSI» повела б на `.exe`.

### 6.7 Конфіг сайту під підкаталог Pages

```ts
// vite.config.ts
base: process.env.GITHUB_PAGES ? '/<REPO>/' : '/',
```

```html
<!-- index.html: тільки %BASE_URL%, ведучий слеш веде в корінь домену -->
<link rel="icon" type="image/png" href="%BASE_URL%favicon.png" />
```

```ts
// рантайм-запити: відносний шлях зламається при заході без слеша в кінці
fetch(`${import.meta.env.BASE_URL}download-manifest.json`)
```

Для `import.meta.env` потрібен `src/vite-env.d.ts` з `/// <reference types="vite/client" />`,
інакше `tsc` падає з `TS2339`.

Перевірка після збірки: у `dist/index.html` **усі** `src` і `href` мусять починатися з `/<REPO>/`.

---

## 7. Принципи просування автора

Це не косметика, а вимога власника. Зберігати в кожному проєкті.

**Мета:** привести людину на особистий хаб `https://spiriturban.github.io/` — там про автора та
його продукти й послуги.

### Класифікація поверхонь

| Тип | Приклади | Що доречно |
|---|---|---|
| **Куди людина приходить сама** | Settings → About, футер сайту, кінець README | ім'я + посилання на хаб |
| **Що бачить один раз** | порожній стан при першому запуску | один тихий рядок |
| **Що завжди на екрані, але поза робочою зоною** | футер бічної панелі | пункт навігації |
| **Що працює без неї** | властивості файлу, Open Graph, поля `package.json` | метадані |

### Заборонено

Банери поверх роботи, тости, модалки, «поставте зірочку» під час використання, згадки в заголовку
вікна, на картках даних чи в тулбарі. Усе, що перериває — дає зворотний ефект.

### Еталонне рішення, яке власник схвалив

Пункт у **футері бічної панелі, одразу під `Settings`**. Стилізований як елемент навігації, а не
як промо-блок: погляд його помічає, натиснути хочеться, заважати не може.

```tsx
<button
  onClick={() => openExternal(PRODUCT_METADATA.authorUrl)}
  title={`More projects and services by ${PRODUCT_METADATA.author}`}
  className="group mt-1 w-full flex items-center gap-2.5 px-3 py-2 rounded-lg
             text-[11px] font-medium text-slate-500
             hover:text-slate-200 hover:bg-slate-800/60 transition-all"
>
  <Sparkles className="w-3.5 h-3.5 text-indigo-400/70 group-hover:text-indigo-400 shrink-0" />
  <span className="truncate">More by {PRODUCT_METADATA.author}</span>
  <ExternalLink className="w-3 h-3 ml-auto opacity-0 group-hover:opacity-60 shrink-0" />
</button>
```

Виміряні параметри, які роблять його ненав'язливим: 11px, приглушений сірий текст, **прозорий
фон**, кольоровий лише значок (індиго на 70%), стрілка зовнішнього посилання з'являється тільки
при наведенні, жодних бейджів і анімацій.

### Обов'язковий мінімум у кожному проєкті

1. `LICENSE` — MIT з **реальним ім'ям автора**. Це не формальність: MIT працює через обов'язок
   зберігати цей рядок у копіях, і якщо там написано «Contributors», механізм авторства не працює.
2. Спільний модуль метаданих з полями `author`, `authorUrl` (хаб), `authorGithubUrl`, `copyright` —
   щоб ім'я задавалося в одному місці.
3. Футер бічної панелі — код вище.
4. Settings → About: ім'я + «more projects and services».
5. Порожній стан: один рядок 11px.
6. Футер сайту: «Built by <ім'я>» → хаб.
7. `<head>` сайту: `author`, `rel="author"`, повний набір Open Graph і Twitter Card. **Це найцінніше
   з усього списку** — кожен репост посилання несе назву, опис і згадку автора, і працює само.
8. `tauri.conf.json`: `bundle.publisher` і `bundle.copyright` з іменем автора — потрапляють у
   властивості файлу й у «Програми та компоненти».
9. Поле `author` у всіх `package.json`.
10. README: розділ Author з описом того, чим автор займається, і посиланням на хаб.

### Про ліцензію

Безкоштовне використання зі збереженням авторства = **MIT**, нічого міняти не треба. Некомерційні
ліцензії (CC BY-NC, PolyForm) для цієї мети шкідливі: це не open source, закриває шлях у Homebrew,
AUR і Debian, відлякує контриб'юторів, а «некомерційне» юридично розмите. Репутація будується з
кількості людей, які користуються.

---

## 8. Варіант Python + React

Що з цього документа переноситься **без змін**:

- сайт на Pages, `base`, `%BASE_URL%`, маніфест завантажень (розділи 6.3, 6.6, 6.7);
- деплой сайту залежною джобою після релізу, правило 14;
- синхронізація версій і `validate-version`;
- шаблон анотацій, правила 20–23;
- **увесь розділ 7** — принципи просування, з поправкою на те, де в застосунку футер навігації.

Що **не переноситься і вимагає окремого проєктування**:

- збірка й пакування: замість `tauri-action` — PyInstaller, Briefcase чи інше; матриця платформ
  лишається, але кроки інші;
- автооновлення: у Tauri воно вбудоване, у Python-стеку його треба або будувати самому, або
  відмовитись. **Якщо застосунок вебовий — апдейтера немає взагалі**, і розділи про ключі,
  `latest.json` і `createUpdaterArtifacts` не застосовуються;
- правила 1, 2, 6, 7 стосуються Rust і зникають.

**Чесно про межі:** Python-варіант цим досвідом **не перевірявся**. Переноситься те, що не
залежить від стеку. Кроки збірки треба спроєктувати заново і провалідувати ручним запуском
(фаза F, крок 3) до першого тега.

---

## 9. Протокол перевірки

Ніколи не заявляти «працює», не отримавши одну з цих відповідей.

```bash
# статус ранів і джоб (без авторизації; ліміт 60 запитів/год на IP)
curl -s "https://api.github.com/repos/<OWNER>/<REPO>/actions/runs?per_page=3"
curl -s "https://api.github.com/repos/<OWNER>/<REPO>/actions/runs/<RUN_ID>/jobs"

# анотації впалої джоби — саме тут буде текст помилки
curl -s "https://api.github.com/repos/<OWNER>/<REPO>/check-runs/<JOB_ID>/annotations"

# ліміт вичерпано?
curl -s "https://api.github.com/rate_limit"

# ендпоінт апдейтера
curl -s -L "https://github.com/<OWNER>/<REPO>/releases/latest/download/latest.json"

# кожне посилання завантаження мусить дати 206
curl -s -o /dev/null -w '%{http_code}\n' -L -r 0-0 \
  "https://github.com/<OWNER>/<REPO>/releases/download/<TAG>/<FILE>"

# версія на сайті
curl -s "https://<OWNER>.github.io/<REPO>/download-manifest.json"
```

Коли API під лімітом — публічні HTML-сторінки ранів читаються без обмежень.

**Реліз вважати завершеним лише коли `latest.json` містить усі очікувані платформні ключі.**
Кожна джоба матриці спершу вивантажує інсталятори і **аж потім** дописує свої записи в маніфест,
тому «файл качається» настає раніше, ніж «оновлення доступне для цієї платформи». Проміжний стан
виглядає як повний реліз: усі інсталятори на місці, а windows-записів у маніфесті ще немає — і
Windows-клієнти оновлення не побачать. Для чотирьох платформ очікувати 11 ключів:

```bash
curl -s -L "https://github.com/<OWNER>/<REPO>/releases/download/<TAG>/latest.json" \
  | python -c "import json,sys; d=json.load(sys.stdin); print(len(d['platforms']), sorted(d['platforms']))"
```

**Іконки перевіряти вмістом:**

```bash
python -c "import struct;d=open('icons/icon.ico','rb').read();print(len(d), struct.unpack('<H',d[4:6])[0],'images')"
```

Правильно: `icon.ico` — кілька зображень і ~19 КБ, `icon.icns` — заголовок `icns` і ~100 КБ,
`icon.png` — 512×512.

---

## 10. Відоме нерозв'язане

**Деплой сайту з реліз-рану може відзвітувати `success`, але його контент не стане живим.**
Спостерігалося один раз: запис деплою з тега був активним і успішним, деплой з `main` —
неактивним, а всі файли сайту віддавалися з попереднього деплою. Причину встановити не вдалося:
`/repos/.../pages` і логи джоби без авторизації закриті. Гіпотезу про гонку з публікацією релізу
перевірено й відкинуто — реліз опублікували за три хвилини до генерації маніфесту.

**Що зроблено:** маніфест тепер запитує конкретний тег замість `latest`, а резольвлена версія
друкується як `::notice` (публічна анотація). Наступного разу буде видно, що саме побачив скрипт.

**Обхід, якщо повториться:** `Actions → Deploy GitHub Pages → Run workflow`, або будь-який пуш,
що зачіпає шляхи з `paths`-фільтра.

---

## 11. Чекліст перед першим тегом

Код:

- [ ] `Cargo.lock` закомічений
- [ ] у десктоп-пакеті є скрипт `"tauri": "tauri"`
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, тести — зелені локально
- [ ] повна локальна збірка проходить
- [ ] іконки справжні (перевірено **вмістом**), `installerIcon` заданий
- [ ] у матриці немає `macos-13`, ключі без дефісів
- [ ] секрети підпису на рівні джоби
- [ ] `includeUpdaterJson: true`, `createUpdaterArtifacts: true`
- [ ] `tagName` під умовою `startsWith(github.ref, 'refs/tags/')`
- [ ] у Rust-джобі CI є збірка фронтенду
- [ ] `pages.yml` має `workflow_call`, `release.yml` — залежну джобу деплою
- [ ] жодного захардкодженого імені артефакта чи версії
- [ ] анотації налаштовані в кожному кроці, що може впасти

GitHub (розділ 5):

- [ ] білінг активний
- [ ] Pages з джерелом GitHub Actions
- [ ] в оточенні дозволені `main` **і** тег `v*.*.*`
- [ ] обидва секрети підпису додані
- [ ] справжній `pubkey` у конфізі

Присутність автора (розділ 7):

- [ ] `LICENSE` з реальним іменем
- [ ] модуль метаданих з `author` / `authorUrl`
- [ ] футер бічної панелі, Settings → About, порожній стан
- [ ] футер сайту, Open Graph у `<head>`
- [ ] `publisher` і `copyright` у бандлі
- [ ] README з розділом Author

Останнє:

- [ ] ручний запуск релізу пройшов зелено (збірка без публікації)
