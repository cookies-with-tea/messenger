# 💬 Messenger — Vue 3 + TypeScript + Tailwind + WebSockets

Real-time chat app with WebSocket communication.

## Project Structure

```
messenger/
├── client/                  # Vue 3 + TS + Tailwind frontend
│   ├── src/
│   │   ├── components/
│   │   │   ├── ChatHeader.vue       # Top bar: status, username editor
│   │   │   ├── ConnectionBadge.vue  # Online/offline indicator
│   │   │   ├── MessageBubble.vue    # Single message with reactions
│   │   │   ├── MessageInput.vue     # Textarea + send button
│   │   │   ├── MessageList.vue      # Scrollable messages list
│   │   │   ├── TypingIndicator.vue  # "User is typing..." bar
│   │   │   ├── UserAvatar.vue       # Emoji avatar with color ring
│   │   │   └── UserList.vue         # Online users sidebar
│   │   ├── composables/
│   │   │   └── useWebSocket.ts      # WS hook: connect/emit/on/reconnect
│   │   ├── stores/
│   │   │   └── chat.ts              # Pinia store: users, messages, typing
│   │   ├── types/
│   │   │   └── index.ts             # All TypeScript interfaces
│   │   ├── App.vue                  # Root: wires WS → store → components
│   │   └── main.ts
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── tsconfig.json
└── server/
    ├── index.js                     # Node.js WebSocket server (no deps except ws)
    └── package.json
```

## Features

- ✅ Real-time messaging via WebSocket
- ✅ Auto-reconnect on disconnect
- ✅ Live typing indicators
- ✅ Emoji reactions on messages (👍❤️😂😮😢🔥)
- ✅ Online users sidebar with live join/leave
- ✅ Click-to-edit username
- ✅ Message history (last 50 on connect)
- ✅ Date separators in chat
- ✅ System messages (join/leave/rename)
- ✅ Pinia store for state management
- ✅ Full TypeScript types

## Quick Start

### 1. Start the WebSocket server

```bash
cd server
npm install
npm start
# ✅ WebSocket server running on ws://localhost:8080
```

### 2. Start the Vue client

```bash
cd client
npm install
npm run dev
# → http://localhost:5173
```

Open multiple browser tabs to test multi-user chat!

### Custom WS URL

Copy `.env.example` to `.env` and set:
```
VITE_WS_URL=ws://your-server:8080
```
