# Sâm Lốc Online - Backend Server

Máy chủ xử lý API HTTP (Authentication) và real-time game lobby (WebSockets) cho trò chơi bài **Sâm Lốc Online**, được phát triển bằng ngôn ngữ **Rust** bảo mật và hiệu năng cao.

## 🛠️ Công Nghệ Sử Dụng

- **Ngôn ngữ**: [Rust (Edition 2024)](https://www.rust-lang.org/)
- **HTTP Web Server**: [Axum v0.8](https://github.com/tokio-rs/axum) - Web framework async nhẹ và hiệu năng cao.
- **WebSocket**: [tokio-tungstenite v0.29](https://github.com/snapview/tokio-tungstenite) - Xử lý truyền nhận tin nhắn real-time giữa client và server.
- **Async Runtime**: [Tokio v1.5](https://tokio.rs/) - Hệ thống async task manager chính.
- **Database**: [PostgreSQL](https://www.postgresql.org/) + [SQLx v0.8](https://github.com/launchbadge/sqlx) - Truy vấn cơ sở dữ liệu async không cần ORM nặng, hỗ trợ compile-time type-checked SQL queries.
- **Authentication & Security**:
  - `jsonwebtoken` - Mã hóa JWT cho các session đăng nhập.
  - `bcrypt` - Mã hóa và kiểm tra password an toàn.
- **Configuration**: `dotenvy` - Đọc thông số cấu hình từ file `.env`.

---

## 🏗️ Cấu Trúc Dự Án

```text
sam-loc-server/
├── migrations/          # File SQL migration để khởi tạo database
├── src/
│   ├── app_state/       # Quản lý global state (sessions, rooms, id generator)
│   ├── auth/            # Xử lý JWT token và hash password (bcrypt)
│   ├── database/        # Kết nối DB & CRUD thao tác với Users
│   ├── helper/          # Các hàm phụ trợ
│   ├── http/            # Axum HTTP routes (Login, Register) & middleware
│   ├── message/         # Struct định nghĩa định dạng tin nhắn trao đổi
│   ├── network/         # WebSocket server engine & Connection handler
│   ├── player/          # Quản lý phiên kết nối (Player Session)
│   ├── room/            # Logic quản lý phòng chơi, luật chơi bài Sâm Lốc
│   ├── lib.rs           # Thư viện core
│   └── main.rs          # Điểm khởi chạy chính (Http & Ws Server chạy song song)
├── .env                 # Cấu hình biến môi trường
├── Cargo.toml           # Định nghĩa dependencies và package metadata
└── Cargo.lock           # Lockfile quản lý phiên bản dependencies
```

---

## ⚙️ Hướng Dẫn Cài Đặt & Khởi Chạy

### 1. Chuẩn Bị Môi Trường
Yêu cầu hệ thống đã cài đặt:
- **Rust toolchain** (phiên bản từ 1.80 trở lên).
- **PostgreSQL** đang hoạt động.
- Cài đặt công cụ SQLx CLI để chạy migration:
  ```bash
  cargo install sqlx-cli --no-default-features --features postgres
  ```

### 2. Cấu Hình Biến Môi Trường (`.env`)
Tạo file `.env` ở thư mục gốc của server (nếu chưa có) và thiết lập các biến sau:
```env
# Địa chỉ socket cho WebSocket Server
WS_HOST=127.0.0.1:8080

# Địa chỉ socket cho HTTP Server
HOST=127.0.0.1:8081

# Chuỗi kết nối tới cơ sở dữ liệu PostgreSQL
DATABASE_URL=postgres://username:password@localhost:5432/sam_loc?sslmode=disable
```

### 3. Tạo Cơ Sở Dữ Liệu & Chạy Migration
Khởi tạo database và bảng `users` thông qua SQLx CLI:
```bash
# Tạo database mới dựa trên DATABASE_URL cấu hình
sqlx database create

# Chạy tất cả file migrations trong thư mục /migrations
sqlx migrate run
```

### 4. Khởi Chạy Server
Khởi động song song cả HTTP server (Port 8081) và WebSocket server (Port 8080):
```bash
cargo run
```
Sau khi chạy thành công, log console sẽ thông báo:
```text
WebSocket server listen at ws://127.0.0.1:8080
```

---

## 📡 Chi Tiết APIs & WebSockets

### 1. HTTP APIs (Port 8081)

#### Đăng ký tài khoản (`POST /register`)
- **Body (JSON)**:
  ```json
  {
    "username": "my_username",
    "password": "my_secure_password"
  }
  ```
- **Response**: Trả về text `"ok"` nếu thành công.

#### Đăng nhập (`POST /login`)
- **Body (JSON)**:
  ```json
  {
    "username": "my_username",
    "password": "my_secure_password"
  }
  ```
- **Response (JSON)**:
  ```json
  {
    "res_type": "login",
    "token": "eyJhbGciOiJIUzI1NiIsIn...",
    "user_id": 1,
    "username": "my_username"
  }
  ```

### 2. WebSocket Connection (Port 8080)
- Điểm kết nối: `ws://127.0.0.1:8080`
- Dùng để đồng bộ trạng thái phòng chơi (lobby), nhận bài, đánh bài và truyền tin nhắn real-time giữa các người chơi trong cùng một bàn đấu bài Sâm Lốc.
