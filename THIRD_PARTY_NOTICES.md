# Thông Báo Bên Thứ Ba (Third-Party Notices)

File này chứa license và thông báo của phần mềm bên thứ ba được sử dụng trong BizClaw.
Một số mã nguồn đã được port, chuyển đổi, hoặc tích hợp trực tiếp từ các dự án này.
License gốc được trích dẫn đầy đủ bên dưới theo yêu cầu.

---

## 1. zca-js

**Nguồn:** https://github.com/RFS-ADRENO/zca-js  
**Sử dụng:** Giao thức Zalo API cá nhân — port từ JavaScript sang Rust.  
**Các file liên quan:** `crates/bizclaw-channels/src/zalo/client/` (xác thực, nhắn tin, bạn bè, nhóm, mã hóa, models)

```
MIT License

Copyright (c) 2023 RFS-ADRENO

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 2. llama.cpp

**Nguồn:** https://github.com/ggerganov/llama.cpp  
**Sử dụng:** Engine suy luận LLM trên thiết bị, tích hợp qua FFI bindings và
dưới dạng Android submodule cho Brain Engine và tính năng AI mobile.  
**Các file liên quan:** `crates/bizclaw-brain/src/llamacpp.rs`, `android/llama.cpp/`

```
MIT License

Copyright (c) 2023-2024 Georgi Gerganov

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## Ghi Chú

Tất cả các nguồn cảm hứng và tham chiếu bên thứ ba khác được ghi nhận trong
[CREDITS.md](CREDITS.md). Các dự án liệt kê ở đó ảnh hưởng đến thiết kế
và kiến trúc của BizClaw nhưng không chứa mã nguồn được sao chép trực tiếp
cần trích dẫn license.
