# Xin 编程语言设计文档

**日期**: 2026-03-13

## 1. 项目概述与目标

### 1.1 项目定位

Xin 是一种静态编译、静态类型的系统编程语言，结合 Rust 的内存安全保证和 Go 的语法简洁性，无运行时 GC，编译为原生机器码。

### 1.2 核心设计理念

1. **内存安全，心智轻松** - 编译期检查 + 智能指针，无需手动管理内存，也无需承受 GC 停顿
2. **语法友好，学习曲线平缓** - 减少 Rust 那种复杂的生命周期标注，让内存安全变得自然
3. **空安全默认** - 变量默认不可空，可空类型显式标记，安全导航操作符避免空指针异常
4. **不可变优先** - 变量和对象默认不可变，显式 `mut` 标记可变性，减少意外修改

### 1.3 MVP 目标

实现一个能编译并运行以下程序的编译器：

```xin
// fibonacci.xin
struct User {
    name: string
    age: int

    func greet() string {
        return "Hello, " + self.name
    }
}

func fibonacci(n: int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

func getPositiveUser(id: int) User? {
    if id > 0 {
        return User { name: "Alice", age: 30 }
    }
    return null
}

func main() {
    // 基础运算
    let result = fibonacci(10)
    println(result)

    // 结构体与方法
    let u = User { name: "Bob", age: 25 }
    println(u.greet())

    // 空安全
    let user = getPositiveUser(1)
    println(user?.name ?? "Unknown")

    // 列表操作
    let list: List<int> = [1, 2, 3, 4, 5]
    for (item in list) {
        println(item)
    }

    // 文件读取
    std.fs.readFile("input.txt", reader: FileReader) {
        for (reader.hasNextLine()) {
            println(reader.readLine())
        }
    }
}
```

## 2. 语法规范

### 2.1 词法元素

**关键字**：
```
let var func struct interface implements if else for in return null mut pub import as true false
```

**类型关键字**：
```
int float bool string void
```

**运算符**：
```
算术: + - * / %
比较: == != < > <= >=
逻辑: && || !
空安全: ?. ??
赋值: = += -= *= /= %=
其他: : -> ( ) [ ] { } , ; . * ?
```

### 2.2 类型系统

**基础类型**：
```
int       // 64位有符号整数
float     // 64位浮点数
bool      // 布尔值
string    // UTF-8 字符串
void      // 无返回值
```

**复合类型**：
```
List<T>        // 列表接口（动态大小）
Map<K, V>      // 哈希表
T[]            // 固定大小数组（栈分配或堆分配，取决于上下文）
User           // 结构体类型
*User          // 指针类型（堆分配，引用计数）
*mut User      // 可变指针
User?          // 可空类型（语法糖，等同于 Option<User>）
Option<T>      // 可选值类型
func(int, int) int  // 函数类型
```

**类型关系说明**：
- `List<T>` 是接口类型，`ArrayList<T>` 是其默认实现
- `T[]` 是固定大小数组，与 `List<T>` 不同，不支持动态添加/删除
- `T?` 是 `Option<T>` 的语法糖

### 2.3 变量声明

```xin
// 不可变变量（默认）
let a = 10
let name: string = "hello"

// 简写语法（等效于 let）
a := 10             // 等效于 let a = 10
name := "hello"     // 类型自动推断

// 可变变量
var count = 0
var items: List<int> = [1, 2, 3]

// 可空变量
let user: User? = null
user := getPositiveUser(1)   // 简写，类型推断

// 变量可变性 vs 对象可变性
let u1 = User { name: "Alice", age: 30 }        // 变量不可变，对象不可变
var u2 = mut User { name: "Bob", age: 25 }     // 变量可变，对象可变

u3 := User { name: "Peter", age: 18 }          // 变量不可变，对象不可变
let u4 = mut User { name: "Xxx", age: 20 }     // 变量不可变，对象可变
u5 := mut User { name: "Yyy", age: 23 }        // 变量不可变，对象可变
```

**声明语法总结**：

| 语法 | 说明 |
|-----|------|
| `let x = 0` | 标准 let 声明 |
| `x := 0` | 简写，等效于 `let x = 0` |
| `var x = 0` | 声明可变变量 |

**可变性规则总结**：

| 声明方式 | 变量是否可重新赋值 | 对象字段是否可修改 |
|---------|-----------------|-----------------|
| `let u = User{}` | ❌ | ❌ |
| `let u = mut User{}` | ❌ | ✅ |
| `var u = User{}` | ✅ | ❌ |
| `var u = mut User{}` | ✅ | ✅ |

### 2.4 哈希表

```xin
// 哈希表字面量
h1 := {a: 1, b: 2, c: 4}              // 值不可变哈希表
h2 := mut {a: "a", b: "b"}            // 值可变哈希表

// 类型标注
let scores: Map<string, int> = {"Alice": 95, "Bob": 87}

// 访问元素
let score = scores["Alice"]

// 修改元素（仅可变哈希表）
var data = mut {x: 1, y: 2}
data["x"] = 10                        // OK
h1["a"] = 100                         // 错误：不可变哈希表

// 添加新键（仅可变哈希表）
data["z"] = 3
```

**与结构体的区分**：

| 语法 | 类型 |
|-----|------|
| `TypeName { field: value }` | 结构体实例化 |
| `{ key: value }` | 哈希表字面量 |

### 2.5 函数定义

```xin
// 标准函数（返回类型直接跟在参数后）
func add(a: int, b: int) int {
    return a + b
}

// 无返回值（可省略 void）
func greet(name: string) {
    print("Hello, " + name)
}

// 显式 void（也可以写）
func greet2(name: string) void {
    print("Hello, " + name)
}

// 单行函数
func double(n: int) int -> n * 2

// 单行无返回值
func log(msg: string) -> print(msg)

// 可变参数
func increment(var n: int) int {
    n = n + 1
    return n
}
```

### 2.6 结构体

```xin
struct User {
    name: string
    age: int

    func greet() string {
        return "Hello, " + self.name
    }
}

// 创建实例
let u1 = User { name: "Alice", age: 30 }      // 不可变
var u2 = mut User { name: "Bob", age: 25 }    // 可变
```

### 2.7 接口

```xin
// 列表接口
interface List<T> {
    mut func add(item: T)
    func get(index: int) T
    func len() int
    func isEmpty() bool
}

// 实现接口
struct ArrayList<T> implements List<T> {
    data: T[]      // 数组类型

    mut func add(item: T) {
        // 添加元素
    }

    func get(index: int) T {
        return self.data[index]
    }

    func len() int {
        return self.data.len()
    }

    func isEmpty() bool {
        return self.len() == 0
    }
}

// 可关闭资源接口
interface Closable {
    func close()
}
```

### 2.8 集合类型

```xin
// List 类型可以用 [ ] 语法创建（默认实现为 ArrayList）
let list: List<int> = [1, 2, 3, 4]

// 通过类型 + [ ] 语法指定具体实现类型
let arrayList = ArrayList[1, 2, 3, 4]
let linkedList = LinkedList[1, 2, 3, 4]

// 空列表
let empty: List<int> = []
let emptyArr = ArrayList[]

// 固定大小数组
let arr: int[] = [1, 2, 3, 4, 5]

arr.len()                  // 长度
arr[0]                     // 索引访问
arr.get(0)                 // 安全获取: Option<int>
```

## 3. 控制流与表达式

### 3.1 条件语句

```xin
// 基本 if-else
if a > b {
    print("a is greater")
} else if a < b {
    print("b is greater")
} else {
    print("equal")
}

// 条件表达式（三目运算符）
let max = a > b ? a : b

// if 作为表达式（返回值）
let result = if a > 0 {
    "positive"
} else {
    "non-positive"
}
```

### 3.2 循环语句

```xin
// 类 C 的 for 循环
for (let i = 0; i < 10; i = i + 1) {
    print(i)
}

// 类 Go 的 for-in 循环
for (item in list) {
    print(item)
}

// 类 Go 的条件循环
for (i < 100) {
    i = i + 1
}

// 无限循环
for {
    if shouldBreak {
        break
    }
}
```

### 3.3 空安全操作

```xin
// 可空类型声明
let name: string? = null
let user: User? = getPositiveUser(1)

// 安全导航操作符
let age = user?.age              // 如果 user 为 null，返回 null
let city = user?.address?.city   // 链式安全导航

// Elvis 操作符
let display = user?.name ?? "Unknown"   // 如果左侧为 null，返回右侧值

// 强制解包（危险操作，需谨慎使用）
let value = user!!.name          // 如果 user 为 null，运行时 panic
```

### 3.4 指针操作

**内存分配语义**：

当声明指针类型时，编译器自动在堆上分配内存并返回引用：

```xin
// 指针声明（自动堆分配）
let p: *User = User { name: "Alice", age: 30 }
// 等价于：在堆上分配 User 对象，p 指向该对象

let mp: *mut User = mut User { name: "Bob", age: 25 }
// 可变指针指向可变对象

// 值类型 vs 指针类型
let u1 = User { name: "A", age: 10 }     // 栈分配（值类型）
let u2: *User = User { name: "B", age: 20 }  // 堆分配（指针类型）

// 指针用于共享和长生命周期场景
let shared: *User = u1                   // 错误：值类型不能直接赋给指针
let shared: *User = User { name: "C", age: 30 }  // 创建新的堆分配
```

**指针使用**：

```xin
// 指针使用（自动解引用）
print(p.name)                    // 自动解引用
print(p.age)
p.greet()                        // 调用方法

// 修改指针指向的对象（需可变指针）
mp.age = 30                      // OK：mp 是 *mut User
p.age = 30                       // 错误：p 是 *User（不可变）

// 函数参数中的指针
func modify(u: *mut User) void {
    u.age = 100                  // OK：可以修改
}

func read(u: *User) void {
    print(u.age)                 // OK：可以读取
    u.age = 100                  // 错误：不可修改
}
```

**指针类型内存管理**：

| 类型 | 内存位置 | 管理方式 |
|-----|---------|---------|
| `User`（值类型） | 栈 | 作用域结束自动释放 |
| `*User`（不可变指针） | 堆 | 引用计数，计数归零时释放 |
| `*mut User`（可变指针） | 堆 | 引用计数，计数归零时释放 |

### 3.5 运算符优先级（从高到低）

```
()  []  .  ?.  !!
*  /  %
+  -
<  >  <=  >=
==  !=
&&
||
?:  ??
=  +=  -=  *=  /=  %=
```

## 4. 类型系统与内存安全

### 4.1 错误处理

**Option<T> 类型**：

`Option<T>` 表示一个值可能存在或不存在。`T?` 是 `Option<T>` 的语法糖。

```xin
// Option<T> 定义（内置类型）
enum Option<T> {
    Some(T)
    None
}

// 使用可空类型
let name: string? = null           // 等同于 Option<string>.None
let age: int? = 25                 // 等同于 Option<int>.Some(25)

// 安全解包
if (name != null) {
    print(name)                    // 此处 name 自动解包为 string
}

// Elvis 操作符
let displayName = name ?? "Unknown"   // 如果 name 为 null，返回 "Unknown"

// 安全导航操作符
let user: User? = getUser()
let age = user?.age                 // 返回 int?

// 强制解包（运行时 panic）
let value = user!!.name             // 如果 user 为 null，程序 panic
```

**Panic 机制**：

当程序遇到不可恢复的错误时，会触发 panic：

```xin
// 强制解包 null 值
let x: int? = null
let y = x!!                         // panic: called !! on a null value

// 数组越界
let arr = [1, 2, 3]
let item = arr[10]                  // panic: index out of bounds: 10 >= 3

// 显式 panic
panic("something went wrong")       // 手动触发 panic
```

**Panic 信息**：
```
thread 'main' panicked at 'called !! on a null value'
  --> src/main.xin:5:13
   |
5  |     let y = x!!
   |             ^^
```

### 4.2 类型检查规则

**基础类型转换**：
```xin
// 隐式转换：无（类型安全）
let a: int = 10
let b: float = a      // 错误：需要显式转换

// 显式转换
let b: float = float(a)
let c: int = int(3.14)    // c = 3（截断）
```

**可空性检查**：
```xin
let name: string = "hello"
let maybe: string? = null

let x: string = maybe    // 错误：不能将可空类型赋给不可空类型
let y: string = maybe!   // 错误：!! 是运行时操作，类型检查通过
let z: string = maybe ?? "default"  // OK：Elvis 确保非空

// 函数返回值
func getName() string? {
    return null
}

let name: string = getName()        // 错误
let name: string = getName() ?? ""  // OK
```

### 4.3 所有权与生命周期

**所有权转移规则**：

所有所有权转移都必须显式使用 `move` 关键字：

```xin
// 值类型所有权转移
let u1 = User { name: "Alice", age: 30 }
let u2 = u1              // 错误：缺少 move 关键字
let u2 = move u1         // 正确：显式转移所有权，u1 不再可用

// 可变引用所有权转移
let m1: *mut User = mut User { name: "Bob", age: 25 }
let m2 = move m1         // 必须使用 move 关键字
let m3 = m2              // 错误：缺少 move 关键字

// 不可变引用：共享，无需 move
let p1: *User = User { name: "C", age: 30 }
let p2 = p1              // OK：不可变引用是共享的
print(p1.name)           // OK：p1 仍有效
print(p2.name)           // OK：p2 也有效
```

**所有权转移规则总结**：

| 源类型 | 目标类型 | 是否需要 `move` |
|-------|---------|---------------|
| 值类型 `User` | 值类型 `User` | ✅ 必须显式 `move` |
| `*User`（不可变引用） | `*User` | ❌ 共享引用，直接赋值 |
| `*mut User`（可变引用） | `*mut User` | ✅ 必须显式 `move` |

**设计理由**：
- 值类型和可变引用都是"独占所有权"，转移后原变量不可用，显式 `move` 让意图清晰
- 不可变引用是"共享引用"，可以安全复制，无需 `move`

### 4.4 内存管理：编译期 GC

**核心机制**：
编译器在编译期自动追踪每个资源的生命周期，在作用域结束时自动插入释放代码。无运行时 GC，无手动内存管理。

```xin
{
    let u = User { name: "Alice", age: 30 }    // 分配内存
    let p: *User = u                            // 引用

    // 使用 u 和 p
    print(u.name)

}   // ← 编译器在此自动插入释放代码
```

**不同场景的释放时机**：

```xin
// 场景 1：作用域结束
{
    let u = User { name: "A", age: 10 }
}   // ← u 在此处释放

// 场景 2：所有权转移
func take(u: User) void {
    // u 在函数结束时释放
}

let u = User { name: "B", age: 20 }
take(move u)        // 所有权转移给函数参数
                   // u 在函数内部释放

// 场景 3：共享引用（引用计数）
let p1: *User = User { name: "C", age: 30 }
let p2 = p1   // 引用计数 +1

{
    let p3 = p1   // 引用计数 +1
}   // ← p3 离开作用域，引用计数 -1

print(p1.name)    // 仍有效

// 引用计数归零时自动释放
```

**资源释放顺序**：
```xin
{
    let a = ResourceA { ... }
    let b = ResourceB { ... }
    let c = ResourceC { ... }

}   // ← 释放顺序：c → b → a（逆序释放）
```

**编译期 GC vs 运行时 GC**：

| 特性 | 编译期 GC（Xin） | 运行时 GC（Java/Go） |
|-----|----------------|-------------------|
| 释放时机 | 确定性（作用域结束） | 不确定（GC 回收时） |
| 性能开销 | 无运行时开销 | GC 暂停、标记清扫 |
| 内存峰值 | 可预测 | 依赖 GC 调度 |
| 析构控制 | 完全可控 | 无法保证执行时机 |

**自定义析构**：
```xin
struct File {
    handle: int

    func drop() void {
        // 编译器在资源释放时自动调用
        closeFile(self.handle)
    }
}

{
    let f = File { handle: open("test.txt") }
    // 使用文件
}   // ← 自动调用 f.drop()，然后释放内存
```

### 4.5 函数参数传递

**参数传递方式对照表**：

| 参数类型 | 传递方式 | 是否复制 | 函数内可写 | 调用方式 | 调用后原变量 |
|---------|---------|---------|----------|---------|------------|
| `a: int` | 值传递 | ✅ 复制 | ❌ | `test(a)` | ✅ |
| `var a: int` | 值传递 | ✅ 复制 | ✅ | `test(a)` | ✅ |
| `u: User` | 值传递 | ✅ 复制 | ❌ | `test(u)` | ✅ |
| `var u: User` | 值传递 | ✅ 复制 | ✅ | `test(u)` | ✅ |
| `u: mut User` | 值传递 | ✅ 复制，但转移所有权 | ✅ | `test(move u)` | ❌ |
| `var u: mut User` | 值传递 | ✅ 复制，但转移所有权 | ✅ | `test(move u)` | ❌ |
| `u: *User` | 引用传递 | ❌ | ❌ | `read(p)` | ✅ |
| `u: *mut User` | 引用传递 | ❌ | ✅ | `modify(move m)` | ❌ |
| `var u: *mut User` | 引用传递 | ❌ | ✅ | `modify(move m)` | ❌ |

**说明**：
- `mut User` 作为参数类型时，表示传递一个可变对象的副本。虽然是值传递（复制），但需要 `move` 关键字转移所有权，确保调用方知道该变量在调用后不可用。
- 这种设计避免了复制可变对象后出现两个可变副本的混淆。

**参数可变性对照表**：

| 参数类型 | 对象可修改 | 参数可重新赋值 | 调用方式 | 调用后原变量 |
|---------|----------|--------------|---------|------------|
| `a: int` | 不适用 | ❌ | `test(a)` | ✅ |
| `var a: int` | 不适用 | ✅ | `test(a)` | ✅ |
| `u: User` | ❌ | ❌ | `test(u)` | ✅ |
| `var u: User` | ❌ | ✅ | `test(u)` | ✅ |
| `u: mut User` | ✅ | ❌ | `test(move u)` | ❌ |
| `var u: mut User` | ✅ | ✅ | `test(move u)` | ❌ |
| `u: *User` | ❌ | ❌ | `test(p)` | ✅ |
| `u: *mut User` | ✅ | ❌ | `modify(move m)` | ❌ |
| `var u: *mut User` | ✅ | ✅ | `modify(move m)` | ❌ |

**关键字职责**：
- `var` → 控制参数变量本身是否可重新赋值
- `mut` → 控制对象属性是否可修改
- 两者独立，可组合使用

## 5. Lambda 表达式

### 5.1 基本语法

```xin
// 表达式体
let add = (a, b) -> a + b
print(add(1, 2))         // 输出: 3

// 块体
let multiply = (a, b) -> {
    let result = a * b
    return result
}
print(multiply(3, 4))    // 输出: 12

// Lambda 参数类型不可省略（当变量类型省略时）
let f = (a: int, b: int) int -> a + b

// Lambda 参数类型可省略（当变量类型明确时）
let f: func(int, int) int = (a, b) -> a + b

// 无返回值 Lambda（可省略 void）
let log = (msg: string) -> print(msg)
```

### 5.2 函数类型语法

| 语法 | 说明 |
|-----|------|
| `func() void` | 无参数，无返回值 |
| `func(int) int` | 单参数，返回 int |
| `func(int, int) int` | 两参数，返回 int |
| `func(string, int) bool` | 多参数，返回 bool |

### 5.3 Lambda 捕获语义

**捕获规则**：

Lambda 默认捕获外部变量的引用（借用）：

```xin
// 捕获不可变变量（引用捕获）
let factor = 10
let scale = (n: int) int -> n * factor   // 捕获 factor 的引用
print(scale(5))                           // 输出: 50

// 捕获可变变量需要 move 关键字
var counter = 0
let increment = move () -> {              // move 捕获所有权
    counter = counter + 1
    return counter
}
print(increment())                        // 输出: 1
print(increment())                        // 输出: 2
// counter 在此不再可用（已被 move）
```

**捕获方式总结**：

| 外部变量类型 | 默认捕获方式 | 说明 |
|------------|------------|------|
| 不可变变量 `let` | 引用捕获 | Lambda 内可读取，不可修改 |
| 可变变量 `var` | 需要 `move` | 必须显式转移所有权 |
| 指针 `*T` | 引用捕获 | 共享引用，引用计数 +1 |
| 可变指针 `*mut T` | 需要 `move` | 必须显式转移所有权 |

**捕获生命周期**：

```xin
func makeCounter() func() int {
    var count = 0
    return move () -> {                  // 必须使用 move，否则 count 在函数返回后失效
        count = count + 1
        return count
    }
}

let counter = makeCounter()
print(counter())                          // 输出: 1
print(counter())                          // 输出: 2
```

### 5.5 作为函数参数

```xin
// Lambda 作为参数
func test(a: int, b: func(int, int) int) int {
    return b(a, a)
}

test(1, (x: int, y: int) int -> x + y)           // 输出: 2

// Lambda 块体作为参数
test(1, (x: int, y: int) int -> {
    return x * y
})                                  // 输出: 1
```

### 5.6 尾随闭包语法

**情况 1：单一无参尾随闭包**

```xin
func test(callback: func()) {
    callback()
}

// 可省略括号
test {
    print("do something")
}
```

**情况 2：单一有参尾随闭包**

```xin
func test(callback: func(int, int)) {
    callback(1, 2)
}

// 省略分号，但不可省略参数类型
test(a: int, b: int) {
    print(a + b)
}
```

**情况 3：有其他参数 + 有参尾随闭包**

```xin
func test(a: int, b: int, callback: func(int, int)) {
    callback(a, b)
}

// 方式 A：尾随闭包（分号方式，可省略参数类型）
test(1, 2; x, y) {
    print(x + y)
}

// 方式 B：用逗号分隔，不可省略 Lambda 参数类型
test(1, 2, x: int, y: int) {
    print(x + y)
}
```

**尾随闭包语法规则总结**：

| 函数参数 | 闭包参数 | 语法 | Lambda 类型 |
|---------|---------|------|------------|
| 无其他参数 | 无参数 | `test { }` | 无参数 |
| 无其他参数 | 有参数 | `test(a: int, b: int) { }` | 不可省略 |
| 有其他参数 | 无参数 | `test(1, 2) { }` | 无参数 |
| 有其他参数 | 有参数 | `test(1, 2; x, y) { }` | 可省略 |
| 有其他参数 | 有参数 | `test(1, 2, x: int, y: int) { }` | 不可省略 |

## 6. 资源管理

### 6.1 自动资源关闭语法

**语法说明**：

`func(resourceParam: ResourceType) { }` 是资源管理的特殊语法糖：

- `resourceParam: ResourceType` 出现在函数调用的参数位置
- 冒号后跟资源类型，表示该参数将由函数提供
- 后续的 `{ }` 块是处理资源的 Lambda
- Lambda 执行完毕后，资源自动关闭

**等价转换**：

```xin
// 资源管理语法
std.fs.readFile("test.txt", reader: FileReader) {
    for (reader.hasNextLine()) {
        print(reader.readLine())
    }
}

// 等价于
std.fs.readFile("test.txt", (reader: FileReader) -> {
    for (reader.hasNextLine()) {
        print(reader.readLine())
    }
})
// readFile 函数内部会调用 reader.close()
```

**自定义资源管理函数**：

```xin
// 定义支持资源管理的函数
func withFile(path: string, handler: func(FileReader)) void {
    let reader = FileReader { path: path }
    handler(reader)
    reader.close()                    // 确保资源关闭
}

// 使用
withFile("test.txt", f: FileReader) {
    for (f.hasNextLine()) {
        print(f.readLine())
    }
}
```

### 6.2 资源管理示例

```xin
// 文件读取 - 自动关闭
std.fs.readFile("test.txt", reader: FileReader) {
    for (reader.hasNextLine()) {
        print(reader.readLine())
    }
}   // ← 自动关闭文件资源

// 文件写入 - 自动关闭
std.fs.writeFile("output.txt", writer: FileWriter) {
    writer.write("Hello, World!")
}   // ← 自动关闭文件资源

// 网络连接 - 自动关闭
std.net.connect("localhost:8080", conn: Connection) {
    conn.send("Hello")
    let response = conn.receive()
    print(response)
}   // ← 自动关闭连接
```

### 6.3 自定义资源管理

```xin
struct File implements Closable {
    handle: int

    func readLine() string {
        // 读取一行
    }

    func hasNextLine() bool {
        // 是否有下一行
    }

    func close() {
        // 关闭文件
    }
}

// 使用自定义资源
func processFile(path: string) {
    std.fs.openFile(path, f: File) {
        for (f.hasNextLine()) {
            print(f.readLine())
        }
    }
}
```

## 7. 模块系统

### 7.1 模块定义

**单文件模块**：
```xin
// utils.xin
pub func helper() void {
    print("helper")
}

pub struct User {
    name: string
    age: int
}

// 私有函数（默认）
func internalHelper() void {
    print("internal")
}
```

**目录模块**：
```
myModule/
├── mod.xin           // 模块入口
├── utils.xin         // 子模块
└── types.xin         // 子模块
```

```xin
// myModule/mod.xin
pub import utils      // 重导出
pub import types

pub func main() void {
    utils.doSomething()
}
```

### 7.2 导入语法

```xin
// 导入整个模块
import utils

utils.helper()

// 导入特定项
import utils { helper, User }

helper()
let u = User { name: "Alice", age: 30 }

// 导入并重命名
import utils as u

u.helper()

// 导入特定项并重命名
import utils { helper as h }

h()
```

### 7.3 可见性规则

```xin
// 默认私有
func privateFunc() void { }     // 仅模块内可见

// 公开
pub func publicFunc() void { }  // 模块外可见

// 结构体字段可见性
pub struct User {
    name: string                // 私有字段
    pub age: int                // 公开字段
}

// 方法可见性
struct Counter {
    count: int

    func getCount() int {       // 私有方法
        return self.count
    }

    pub func increment() void { // 公开方法
        self.count = self.count + 1
    }
}
```

## 8. 标准库

### 8.1 内置函数

**输入输出**：
```xin
print(value)                    // 打印任意值
println(value)                  // 打印并换行
format("Hello, {}!", name)      // 格式化字符串
readLine()                      // 读取一行输入
```

**类型转换**：
```xin
float(10)                       // int -> float
int(3.14)                       // float -> int（截断）
string(100)                     // int -> string
int("42")                       // string -> int（可能失败）
parseInt("42")                  // 返回 Option<int>
```

### 8.2 字符串操作

```xin
let s = "Hello, World!"

s.len()                         // 字节长度
s.chars()                       // 字符迭代器
s.contains("World")             // 是否包含
s.split(",")                    // 分割
s.trim()                        // 去除空白
s.toUpper()                     // 转大写
s.toLower()                     // 转小写
s + "!"                         // 拼接
s.startsWith("Hello")           // 是否以...开头
s.endsWith("!")                 // 是否以...结尾
```

### 8.3 Map 操作

```xin
let m = {a: 1, b: 2}

m.len()                         // 长度
m.get("a")                      // 安全获取: Option<int>
m["a"]                          // 键访问（不存在则 panic）
m.insert("c", 3)                // 插入（需可变）
m.remove("a")                   // 删除
m.keys()                        // 键迭代器
m.values()                      // 值迭代器
m.containsKey("a")              // 是否包含键
```

### 8.4 标准库模块

**文件 I/O**：
```xin
import std.fs

std.fs.readFile("test.txt", reader: FileReader) {
    for (reader.hasNextLine()) {
        print(reader.readLine())
    }
}
std.fs.writeFile("output.txt", "Hello")
let exists = std.fs.exists("test.txt")
```

**系统调用**：
```xin
import std.os

let home = std.os.env("HOME")
let args = std.os.args()
std.os.exit(0)
```

**时间**：
```xin
import std.time

let now = std.time.now()
let s = std.time.format(now, "%Y-%m-%d %H:%M:%S")
```

### 8.5 标准库模块结构

```
std/
├── prelude.xin       // 自动导入的基础类型和函数
├── fs.xin            // 文件系统
├── os.xin            // 操作系统接口
├── net.xin           // 网络
├── time.xin          // 时间
├── math.xin          // 数学
├── json.xin          // JSON 处理
└── collections/
    ├── mod.xin
    ├── list.xin      // List 接口和实现
    └── map.xin       // Map 类型
```

## 9. 编译器架构

### 9.1 整体架构

```
源码 → Lexer → Parser → AST → Semantic Analysis → IR Generation → IR Optimization → Cranelift → 机器码
```

### 9.2 编译阶段

| 阶段 | 输入 | 输出 | 职责 |
|-----|------|------|------|
| 词法分析 | 源代码字符串 | Token 流 | 识别关键字、标识符、字面量、运算符 |
| 语法分析 | Token 流 | AST | 构建语法结构，识别表达式、语句、声明 |
| 语义分析 | AST | 带类型的 AST | 类型检查、可空性检查、所有权检查 |
| IR 生成 | 带类型的 AST | Xin IR | 降低抽象层次 |
| IR 优化 | Xin IR | 优化后的 IR | 死代码消除、常量折叠、内联 |
| 代码生成 | 优化后的 IR | 机器码 | 通过 Cranelift 生成目标平台代码 |

### 9.3 诊断系统

```
error[E001]: cannot assign to immutable variable `x`
  --> src/main.xin:5:5
   |
5  |     x = 10
   |     ^^^^^^ cannot assign to `x`
   |
help: declare the variable as mutable with `var`
   |
4  |     var x = 5
   |         +++
```

### 9.4 项目目录结构

```
xin/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs               # 库入口
│   ├── lexer/               # 词法分析
│   ├── parser/              # 语法分析
│   ├── semantic/            # 语义分析
│   ├── ir/                  # 中间表示
│   ├── codegen/             # 代码生成
│   ├── diagnostics/         # 诊断系统
│   └── stdlib/              # 标准库
├── tests/
└── docs/
```

## 10. 编译目标

- 编译为原生机器码，直接执行
- 支持 Windows、macOS、Linux 平台
- 可适配多种后端（LLVM 或 Cranelift）
- MVP 阶段优先适配 Cranelift

## 11. MVP 实现路线图

### 11.1 阶段划分

| 阶段 | 内容 |
|-----|------|
| 1 | 基础框架：项目结构、CLI、词法分析器 |
| 2 | 语法解析：AST 定义、Parser、错误恢复 |
| 3 | 语义分析：类型系统、类型检查、作用域、可空性 |
| 4 | 所有权系统：所有权检查、生命周期、Move 语义、借用检查 |
| 5 | IR 与代码生成：IR 定义、Cranelift 集成、基础优化 |
| 6 | 标准库：基础类型方法、文件 I/O、系统调用 |
| 7 | 完善与测试：诊断信息、集成测试、文档 |

### 11.2 MVP 验收标准

能够成功编译并运行本文档开头的 `fibonacci.xin` 程序。

```bash
xin compile fibonacci.xin -o fibonacci
./fibonacci
```