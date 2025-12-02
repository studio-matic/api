## Prerequisites

* Install [Nix](https://nixos.org/download).

## 1. Testing Locally

```bash
nix develop . --no-pure-eval
./start.sh offline
```

First-time DB setup:

```sql
source ./setup.sql
```

## 2. Deployment

```bash
nix develop . --no-pure-eval
./start.sh online
```

First-time DB setup:

```sql
source ./setup.sql
```

### Backend (Fly.io)

```bash
cd ./back
fly deploy               # standard
fly deploy --local-only  # build locally with Docker
```

For local builds make sure the daemon is running with `ides start docker`.

### Frontend (GitHub Pages)

```bash
git subtree push --prefix front origin gh-pages
# or for forced changes
git subtree split --prefix=front -b subtree-temp
git push origin subtree-temp:gh-pages --force
git branch -D subtree-temp
```

## 3. Spin Up

```bash
nix develop . --no-pure-eval
fly scale count 1            # in ./db-fly-io and ./back
fly machine start            # in ./db-fly-io then ./back
./start.sh online
```

## 4. Spin Down

```bash
nix develop . --no-pure-eval
fly scale count 0            # in ./db-fly-io and ./back
```
