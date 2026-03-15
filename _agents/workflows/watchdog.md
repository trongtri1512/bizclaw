---
description: 🐕 Watchdog — Auto-kill hung terminal processes
---

# 🐕 Watchdog — Terminal Process Guard

> **Role**: System Watchdog  
> **Mission**: Tự động kill terminal processes chạy quá 20 phút  
> **When**: Khi terminal bị treo, cargo build quá lâu, hoặc server chạy rồi quên tắt

// turbo-all

---

## Cách dùng

### 1. Quick Status — Xem gì đang chạy
```bash
./scripts/watchdog.sh --status
```

### 2. Auto Kill — Kill processes chạy > 20 phút  
```bash
./scripts/watchdog.sh
```

### 3. Emergency Kill — Kill TẤT CẢ bizclaw processes
```bash
./scripts/watchdog.sh --kill-all
```

### 4. Dry Run — Xem sẽ kill gì mà không kill thật
```bash
./scripts/watchdog.sh --dry-run
```

### 5. Daemon Mode — Chạy nền, tự kiểm tra mỗi 60 giây
```bash
./scripts/watchdog.sh --daemon &
```

---

## Tuỳ chỉnh

| Env Variable | Mặc định | Mô tả |
|---|---|---|
| `WATCHDOG_MAX_MINUTES` | 20 | Thời gian tối đa (phút) |
| `WATCHDOG_INTERVAL` | 60 | Khoảng cách kiểm tra (giây) |
| `WATCHDOG_DRY_RUN` | 0 | 1 = chỉ preview, không kill |

Ví dụ thay đổi timeout:
```bash
WATCHDOG_MAX_MINUTES=30 ./scripts/watchdog.sh
```

---

## Quy trình khi bị treo

1. Chạy `./scripts/watchdog.sh --status` để xem
2. Nếu có process > 20 min → `./scripts/watchdog.sh` (auto kill)
3. Nếu muốn kill hết → `./scripts/watchdog.sh --kill-all`
4. Sau đó restart: `cargo run --bin bizclaw`

---

## Tích hợp với BizClaw

Shell timeout đã được tăng từ 30s → 900s (15 phút):
- File: `crates/bizclaw-tools/src/shell.rs`
- File: `crates/bizclaw-runtime/src/lib.rs`
- Env: `BIZCLAW_SHELL_TIMEOUT_SECS=900`

Watchdog xử lý layer phía ngoài — kill process level, không phụ thuộc vào code bên trong.
