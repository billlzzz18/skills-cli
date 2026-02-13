# Skills CLI

Universal Agent Skills Manager

จัดการ skill ให้ agents-cli ทุกตัวผ่าน binary เดียว  
รองรับ project-local skills และ global skills

---

# 🔹 แนวคิด

ระบบนี้ทำหน้าที่:

- รวม (merge) skill จาก
  - `./skills` (project)
  - `~/.agents/skills` (global)
- ติดตั้ง skill ให้ agent ใดก็ได้
- ใช้ symlink (ไม่ copy)
- ไม่ใช้ hash
- ไม่มี metadata
- ไม่ต้อง register phase

---

# 🔹 รองรับอะไรบ้าง

✔ รองรับ agent ใดก็ได้ (ชื่อ dynamic)  
✔ รองรับ project-local skill  
✔ รองรับ global skill  
✔ project override global ถ้าชื่อชน  
✔ install ทีละ skill  
✔ install ทั้งหมด  
✔ uninstall ทีละ skill  
✔ uninstall ทั้งหมด  
✔ update (relink ใหม่)  
✔ list global + project  
✔ list skill ของ agent  
✔ detect agents ที่มีอยู่  
✔ help ผ่าน clap  

---

# 🔹 โครงสร้างที่ระบบใช้

## Project Skills

```

your-project/
└── skills/
├── foo/
└── bar/

```

## Global Skills

```

~/.agents/skills/

```

## Agent Install Path

```

~/.gemini/skills/
~/.codex/skills/
~/.kilo/skills/
~/.opencode/skills/

```

ระบบจะสร้างโฟลเดอร์ agent ให้อัตโนมัติถ้ายังไม่มี

---

# 🔹 ลำดับการค้นหา Skill

เวลาติดตั้ง:

1. ตรวจ `./skills`
2. ถ้าไม่พบ → ตรวจ `~/.agents/skills`
3. ถ้าไม่พบทั้งคู่ → Error

ถ้ามีชื่อซ้ำ:
- Project จะ override Global

---

# 🔹 คำสั่งทั้งหมด

## ดู skill ทั้งหมด (merged)

```

skills list

```

---

## ดู agents ที่มีอยู่ในเครื่อง

```

skills agents

```

---

## ติดตั้ง skill ให้ agent

```

skills --init <agent> install --skills <skill-name>

```

ตัวอย่าง:

```

skills --init gemini install --skills find-skills

```

---

## ติดตั้งทั้งหมดให้ agent

```

skills --init gemini install --all

```

---

## ลบ skill จาก agent

```

skills --init gemini uninstall --skills find-skills

```

---

## ลบทั้งหมดออกจาก agent

```

skills --init gemini uninstall --all

```

---

## อัปเดต (relink ใหม่)

```

skills --init gemini update

```

---

## ดู skill ที่ agent ติดตั้งอยู่

```

skills --init gemini list

```

---

## ดู help

```

skills --help
skills --init gemini install --help

```

---

# 🔹 ตัวอย่าง Workflow จริง

## 1. สร้าง skill ใน project

```

mkdir -p skills/my-skill

```

## 2. ติดตั้งให้ gemini

```

skills --init gemini install --skills my-skill

```

## 3. ตรวจสอบ

```

ls -l ~/.gemini/skills

```

ควรเห็น:

```

my-skill -> /home/user/project/skills/my-skill

```

---

# 🔹 การติดตั้ง

## ใช้ Cargo

```

cargo install --path .

```

## หรือ build แล้ว copy

```

cargo build --release
sudo cp target/release/skills-cli /usr/local/bin/skills

```

---

# 🔹 ข้อจำกัด

- ใช้ symlink (Linux / macOS ปกติ)
- Windows ต้องเปิด Developer Mode หรือใช้ WSL
- ไม่มี versioning ของ skill
- ไม่มี dependency resolution

---

# 🔹 Release

```

cargo build --release

```

Binary อยู่ที่:

```

target/release/skills-cli

```

---

# 🔹 License

MIT
```

---


