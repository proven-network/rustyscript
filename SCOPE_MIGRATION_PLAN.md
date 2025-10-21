# deno_core Scope API Migration Plan

## Summary of Changes in deno_core

### Key Breaking Changes:
1. **`JsRuntime::handle_scope()` removed** - This method no longer exists
2. **Scope types changed** - `v8::HandleScope` → `v8::PinScope` for most APIs
3. **`v8::Local::new()` signature changed** - Now takes `&scope` instead of `&mut scope`
4. **`v8::Global::new()` signature changed** - Now takes `&isolate` instead of `&mut scope`
5. **`v8::Global::open()` signature changed** - Now takes `&mut isolate` instead of `&mut scope`
6. **TryCatch creation changed** - Use macro instead of direct construction

### New Patterns:

#### 1. Creating a scope from JsRuntime:
```rust
// OLD:
let mut scope = runtime.handle_scope();

// NEW:
deno_core::scope!(scope, runtime);
// This expands to:
// - Get the main context
// - Get the isolate
// - Create a HandleScope on the isolate
// - Create a ContextScope with the context
```

#### 2. Using v8::Local::new() with globals:
```rust
// OLD:
let local = v8::Local::new(&mut scope, &global_value);

// NEW:
let local = v8::Local::new(scope, &global_value);
// Note: Takes &scope, not &mut scope
```

#### 3. Using v8::Global::new():
```rust
// OLD:
let global = v8::Global::new(&mut scope, local_value);

// NEW:
// Need to get the isolate from scope
let global = v8::Global::new(scope, local_value);
// The scope now implements AsRef<Isolate>
```

#### 4. Using v8::Global::open():
```rust
// OLD:
let local = global.open(&mut scope);

// NEW:
let local = global.open(scope);
// But scope must be an isolate or convertible to one
// When using deno_core::scope!, it creates the right type
```

#### 5. TryCatch scopes:
```rust
// OLD:
let tc_scope = &mut v8::TryCatch::new(scope);

// NEW:
v8::tc_scope!(let tc_scope, scope);
```

## Migration Strategy for rustyscript

### Phase 1: Identify All Patterns

1. **`runtime.handle_scope()` calls** - Most common pattern
   - In `js_value/promise.rs`: Lines 36, 61, 70
   - In `runtime.rs`: Lines 495, 568
   - In `inner_runtime.rs`: Line 702

2. **`v8::Local::new()` calls** - Need to check if using correct reference type
   - In `js_value.rs`: Multiple locations
   - Check all uses to ensure `&scope` not `&mut scope`

3. **`v8::Global::new()` calls** - Need proper isolate reference
   - In `js_value.rs`: Need to update scope type

4. **`v8::Global::open()` calls** - Need proper isolate reference
   - In `inner_runtime.rs`: Line 597

5. **TryCatch usage** - Need to use macro
   - In `inner_runtime.rs`: Lines with scope.has_caught(), scope.message()

### Phase 2: Handle Lifetime Issues

The main challenge: **temporary lifetime problems**

When you do:
```rust
let mut scope = deno_core::scope!(scope, runtime.deno_runtime());
```

The macro needs `runtime.deno_runtime()` to live long enough. The temporary from `deno_runtime()` is dropped too soon.

**Solution patterns:**

#### Pattern A: Split into steps
```rust
// Instead of:
let mut scope = some_expr().handle_scope();

// Do:
let runtime = some_expr();
deno_core::scope!(scope, runtime);
```

#### Pattern B: Use a block to control lifetimes
```rust
{
    let runtime = self.deno_runtime();
    deno_core::scope!(scope, runtime);
    // Use scope here
}
```

#### Pattern C: When working with methods, borrow early
```rust
// In a method that takes &mut self:
let rt = self.deno_runtime();
deno_core::scope!(scope, rt);
```

### Phase 3: Specific File Changes

#### `js_value/promise.rs`
Lines 36-38:
```rust
// OLD:
let mut scope = runtime.handle_scope();
let local = v8::Local::new(&mut scope, &result);
Ok(deno_core::serde_v8::from_v8(&mut scope, local)?)

// NEW:
deno_core::scope!(scope, runtime);
let local = v8::Local::new(scope, &result);
Ok(deno_core::serde_v8::from_v8(scope, local)?)
```

Lines 61, 70: Similar pattern in different methods

#### `runtime.rs`
Lines 495, 568:
```rust
// OLD:
let function = function.as_global(&mut self.deno_runtime().handle_scope());

// NEW:
let rt = self.deno_runtime();
deno_core::scope!(scope, rt);
let function = function.as_global(scope);
```

#### `inner_runtime.rs`
Line 597 and surrounding (TryCatch block):
```rust
// OLD:
let mut scope = v8::TryCatch::new(self.deno_runtime().handle_scope());
let function_instance = function.open(&mut scope);
let args = decode_args(args, &mut scope)?;
let result = function_instance.call(&mut scope, namespace, &args);
if let Some(value) = result {
    let value = v8::Global::new(&mut scope, value);
    Ok(value)
} else if scope.has_caught() {
    let e = scope.message()...
}

// NEW:
let rt = self.deno_runtime();
deno_core::scope!(scope, rt);
v8::tc_scope!(let tc_scope, scope);
let function_instance = function.open(scope);
let args = decode_args(args, tc_scope)?;
let result = function_instance.call(tc_scope, namespace, &args);
if let Some(value) = result {
    let value = v8::Global::new(tc_scope, value);
    Ok(value)
} else if tc_scope.has_caught() {
    let e = tc_scope.message()...
}
```

#### `js_value.rs`
Lines with `v8::Local::new()` and `v8::Global::new()`:
- Change `&mut scope` to `scope` for `v8::Local::new()`
- Ensure scope type is correct for `v8::Global::new()`

### Phase 4: Type Signature Updates

Update any function signatures that use:
- `v8::HandleScope<'a>` → `v8::PinScope<'a, 'i>` (add second lifetime)
- Methods that create/use scopes need careful lifetime annotations

### Phase 5: Testing Strategy

After each file:
1. Run `cargo check` on rustyscript
2. Fix any remaining errors
3. Move to next file
4. Once all compile, run full test suite

## Order of Implementation

1. Start with `js_value/promise.rs` - simplest cases
2. Move to `runtime.rs` - method call patterns
3. Tackle `inner_runtime.rs` - most complex (TryCatch)
4. Fix `js_value.rs` - parameter type changes
5. Handle any remaining files with scope usage
6. Final compilation and testing
