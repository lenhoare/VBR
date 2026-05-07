# VBR Standard Library — Implementation Reference

Crate: `vbr_stdlib`  
Path: `vbr_stdlib/`  
Dependencies: `serde`, `serde_json`, `reqwest` (blocking + json features), `chrono`, `regex`

The stdlib wraps common Rust libraries behind a flat, VBA-style API. Everything is a unit struct with static methods — no construction, no state. All fallible operations return `Result<T, String>` so errors stay simple at the teaching level.

---

## FileSystem

```rust
use vbr_stdlib::FileSystem;
```

| Method | Signature | VBA equivalent |
|--------|-----------|----------------|
| `read` | `(path: &str) -> Result<String, String>` | `TextStream.ReadAll` |
| `read_lines` | `(path: &str) -> Result<Vec<String>, String>` | Line-by-line TextStream |
| `write` | `(path: &str, contents: &str) -> Result<(), String>` | `CreateTextFile` + Write |
| `append` | `(path: &str, text: &str) -> Result<(), String>` | `OpenTextFile(ForAppending)` |
| `exists` | `(path: &str) -> bool` | `FSO.FileExists` |
| `copy` | `(source: &str, destination: &str) -> Result<(), String>` | `FSO.CopyFile` |
| `move_file` | `(source: &str, destination: &str) -> Result<(), String>` | `FSO.MoveFile` |
| `delete` | `(path: &str) -> Result<(), String>` | `FSO.DeleteFile` |
| `create_folder` | `(path: &str) -> Result<(), String>` | `FSO.CreateFolder` |
| `create_folder_all` | `(path: &str) -> Result<(), String>` | CreateFolder with parent creation |
| `folder_exists` | `(path: &str) -> bool` | `FSO.FolderExists` |
| `delete_folder` | `(path: &str) -> Result<(), String>` | `FSO.DeleteFolder` |
| `delete_folder_all` | `(path: &str) -> Result<(), String>` | `FSO.DeleteFolder` (recursive) |

```rust
FileSystem::write("output.txt", "hello")?;
let text = FileSystem::read("output.txt")?;
let lines = FileSystem::read_lines("log.txt")?;
```

---

## Json

```rust
use vbr_stdlib::Json;
use serde_json::Value;
```

Backed by `serde_json::Value`. Objects and arrays are created empty and mutated with `set`.

| Method | Signature |
|--------|-----------|
| `parse` | `(text: &str) -> Result<Value, String>` |
| `object` | `() -> Value` |
| `array` | `() -> Value` |
| `to_string` | `(value: &Value) -> Result<String, String>` |
| `to_pretty` | `(value: &Value) -> Result<String, String>` |
| `has_key` | `(value: &Value, key: &str) -> bool` |
| `get_string` | `(value: &Value, key: &str) -> Result<String, String>` |
| `get_int` | `(value: &Value, key: &str) -> Result<i64, String>` |
| `get_float` | `(value: &Value, key: &str) -> Result<f64, String>` |
| `get_bool` | `(value: &Value, key: &str) -> Result<bool, String>` |
| `get_array` | `(value: &Value, key: &str) -> Result<Vec<Value>, String>` |
| `set` | `(value: &mut Value, key: &str, val: Value)` |

```rust
let data = Json::parse(r#"{"name":"Alice","age":42}"#)?;
let name = Json::get_string(&data, "name")?;

let mut obj = Json::object();
Json::set(&mut obj, "active", serde_json::json!(true));
let text = Json::to_pretty(&obj)?;
```

---

## Http

```rust
use vbr_stdlib::{Http, HttpResponse};
```

All methods use `reqwest::blocking` — calls block the current thread, matching how VBA HTTP worked. Use `get_response` when you need status codes or response headers.

| Method | Signature |
|--------|-----------|
| `get` | `(url: &str) -> Result<String, String>` |
| `get_with_headers` | `(url: &str, headers: HashMap<String, String>) -> Result<String, String>` |
| `post` | `(url: &str, body: &str) -> Result<String, String>` |
| `post_json` | `(url: &str, body: &Value) -> Result<String, String>` |
| `get_response` | `(url: &str) -> Result<HttpResponse, String>` |
| `headers` | `() -> HashMap<String, String>` |

**`HttpResponse` methods:**

| Method | Signature |
|--------|-----------|
| `status` | `(&self) -> u16` |
| `text` | `(self) -> Result<String, String>` |
| `header` | `(&self, key: &str) -> Result<String, String>` |

```rust
let body = Http::get("https://api.example.com/data")?;

let mut hdrs = Http::headers();
hdrs.insert("Authorization".into(), "Bearer abc123".into());
let body = Http::get_with_headers("https://api.example.com/me", hdrs)?;

let resp = Http::get_response("https://api.example.com")?;
println!("Status: {}", resp.status());
let body = resp.text()?;
```

---

## DateTime

```rust
use vbr_stdlib::DateTime;
```

Backed by `chrono`. `now()` and arithmetic methods work with `chrono::DateTime<Local>`. `parse` returns `NaiveDateTime` (no timezone) matching the string-in, string-out pattern of VBA's `CDate`.

| Method | Signature |
|--------|-----------|
| `now` | `() -> DateTime<Local>` |
| `utc` | `() -> DateTime<Utc>` |
| `format` | `(dt: &DateTime<Local>, pattern: &str) -> String` |
| `parse` | `(text: &str, pattern: &str) -> Result<NaiveDateTime, String>` |
| `add_days` | `(dt: DateTime<Local>, days: i64) -> DateTime<Local>` |
| `add_hours` | `(dt: DateTime<Local>, hours: i64) -> DateTime<Local>` |
| `add_minutes` | `(dt: DateTime<Local>, minutes: i64) -> DateTime<Local>` |
| `diff_days` | `(dt1: DateTime<Local>, dt2: DateTime<Local>) -> i64` |
| `diff_hours` | `(dt1: DateTime<Local>, dt2: DateTime<Local>) -> i64` |
| `year` | `(dt: &DateTime<Local>) -> i32` |
| `month` | `(dt: &DateTime<Local>) -> u32` |
| `day` | `(dt: &DateTime<Local>) -> u32` |

Format patterns use `chrono`'s strftime syntax: `%Y-%m-%d`, `%H:%M:%S`, etc.

```rust
let now = DateTime::now();
let stamp = DateTime::format(&now, "%Y-%m-%d %H:%M:%S");
let next_week = DateTime::add_days(now, 7);
let days = DateTime::diff_days(now, next_week); // 7
```

---

## Regex

```rust
use vbr_stdlib::Regex;
```

Backed by the `regex` crate. The pattern is compiled fresh on each call — suitable for one-shot use. If you're calling the same pattern in a loop, compile a `regex::Regex` directly instead.

| Method | Signature |
|--------|-----------|
| `is_match` | `(pattern: &str, text: &str) -> Result<bool, String>` |
| `find` | `(pattern: &str, text: &str) -> Result<Option<String>, String>` |
| `find_all` | `(pattern: &str, text: &str) -> Result<Vec<String>, String>` |
| `replace` | `(pattern: &str, text: &str, replacement: &str) -> Result<String, String>` |
| `replace_all` | `(pattern: &str, text: &str, replacement: &str) -> Result<String, String>` |
| `captures` | `(pattern: &str, text: &str) -> Result<Vec<String>, String>` |

`captures` returns the capture group strings (skipping the full match at index 0), matching how VBA's `SubMatches` collection worked.

```rust
let matched = Regex::is_match(r"\d+", "order 42")?;     // true
let first   = Regex::find(r"\d+", "order 42")?;          // Some("42")
let all     = Regex::find_all(r"\d+", "1 and 2 and 3")?; // ["1","2","3"]
let cleaned = Regex::replace_all(r"\s+", "too  many  spaces", " ")?;

let caps = Regex::captures(r"(\w+)@(\w+)", "user@host")?;
// caps[0] = "user", caps[1] = "host"
```

---

## Design notes

**Why static methods on unit structs?**  
VBA programmers are used to calling `FSO.FileExists(path)` and `RegExp.Test(text)` — a dot-separated verb on a named object. The unit struct pattern (`FileSystem::exists(path)`) reads identically and requires no construction or cleanup, easing the mental translation.

**Why `Result<T, String>` and not `Result<T, Box<dyn Error>>`?**  
Keeps error handling at the `?` level without requiring the caller to understand trait objects. For a teaching tool this is the right trade-off; production code would use a typed error enum.

**Blocking HTTP**  
`reqwest::blocking` is intentional. Async HTTP requires `tokio` and ownership patterns that are a separate lesson. VBR introduces one concept at a time.
