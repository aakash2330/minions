# Minions

Basic Phaser 4 frontend and Rust Actix-web backend setup.

## Stack

- Frontend: Vite, React, TypeScript, Phaser 4
- Backend: Rust, Actix-web
- Communication: HTTP, with the frontend dev server proxying `/api` to the backend

Sprite art is exported from the Universal LPC Spritesheet Generator. See
`frontend/public/assets/source/lpc-minionlike-credits.txt` and
`frontend/public/assets/source/lpc-player-credits.txt` for attribution.

## Run

Start the backend:

```sh
cd backend
cargo run
```

Start the frontend:

```sh
cd frontend
bun install
bun run dev
```

Health check:

```sh
curl http://127.0.0.1:8080/health
```

From the frontend dev server, the same backend route is available at:

```text
/api/health
```
