
## Vec As First Class VBR

Declaration:
```vba
Dim scores As New Vec<Long>
Dim names As New Vec<String>
```

---

## The Core Operations

**Adding elements:**
```vba
scores.push(42)
scores.push(100)
```
```rust
scores.push(42);
scores.push(100);
```
Clean direct mapping.

**Removing last element:**
```vba
Dim last As Long = scores.pop()
```
This is interesting — `pop()` returns `Option<T>` in Rust because the Vec might be empty. Consistent with our existing pattern:
```rust
let last = scores.pop().ok_or("Vec is empty")?;
```
Teaching note:
```
ℹ pop() returns nothing if the Vec is empty.
  VBR converts to Result automatically.
  Use .Unwrap() for training wheels or handle with Match.
```

**Length:**
```vba
Dim n As Long = scores.len()
```
```rust
let n = scores.len();
```
Clean mapping.

**Is empty:**
```vba
If scores.is_empty() Then
```
```rust
if scores.is_empty() {
```
Clean mapping.

**Insert at position:**
```vba
scores.insert(2, 42)
```
```rust
scores.insert(2, 42);
```
Clean mapping. Worth a note:
```
ℹ insert() shifts all elements after the position.
  This is an O(n) operation — it gets slower as Vec grows.
```

**Remove at position:**
```vba
Dim removed As Long = scores.remove(2)
```
```rust
let removed = scores.remove(2);
```
Clean mapping. Same performance note as insert.

**Safe access — consistent with arrays:**
```vba
Dim x As Long = scores.get(0)
```
```rust
let x = scores.get(0).ok_or("Index out of bounds")?;
```

---

## Iterating — The Interesting One

```vba
For Each score In scores
    Debug.Print score
Next
```
```rust
for score in &scores {
    println!("{}", score);
}
```
Teaching note — first time only:
```
ℹ For Each on a Vec borrows it with &.
  This means scores still owns its data after the loop.
  You are just looking at each element in turn.
```

---

## Initialising With Values

VBA has no equivalent but this is so useful it should be VBR syntax:

```VBR
Dim scores As Vec<Long> = [1, 2, 3, 4, 5]
```
```rust
let scores = vec![1, 2, 3, 4, 5];
```
Teaching note:
```
ℹ Vec can be initialised with values directly using [].
  This has no VBA equivalent — it is a VBR extension.
  Rust generates a vec![] macro for this.
```

---

## The Complete Vec Picture

| VBR | Rust | Notes |
|---|---|---|
| `Dim x As New Vec<Long>` | `let mut x: Vec<i32> = Vec::new()` | ℹ mut automatic |
| `Dim x As Vec<Long> = [1,2,3]` | `let x = vec![1, 2, 3]` | VBR extension |
| `x.push(value)` | `x.push(value)` | Clean mapping |
| `x.pop()` | `x.pop().ok_or(...)` | Returns Result |
| `x.len()` | `x.len()` | Clean mapping |
| `x.is_empty()` | `x.is_empty()` | Clean mapping |
| `x.insert(i, value)` | `x.insert(i, value)` | ⚠ O(n) operation |
| `x.remove(i)` | `x.remove(i)` | ⚠ O(n) operation |
| `x.get(i)` | `x.get(i).ok_or(...)` | Returns Result |
| `For Each v In x` | `for v in &x` | ℹ Borrows x |
| `x.contains(v)` | `x.contains(&v)` | ℹ reference to v|
---

