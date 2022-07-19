# Library for generating internationalization (i18n) rust module files.

## WARNING
- The library is in a **prototype** state. 
- It does not have a stable interface so, any new version may break backward compatibility.
- Also, it is not presented in the [crates.io](https://crates.io/) at this moment.
- As it is a prototype it does not have a good internal architecture yet.
- It supports text translation only. I.e. it does not support formatting for dates, currency, values etc...  
But it supports argument types which allows you to have workarounds.
- It does not support runtime loading files with translation.


## Features
- Generating code for including into your binary. Actually it generates rust module code.
- `.yml` files for storing translations.
- It is a dependency for compile(build) time only. Your code needs to `use` only generated file which uses only Rust standard and core libraries. (Can be changed in the future)
- Allocation free. Generated functions return structures which implement `std::fmt::Display` instead of `String` so, you are responsible what to do in the next step.
- Translation text variables support types. (see examples)
- Any types including custom ones which implement `std::fmt::Display` can be used as a text variable.
- Text without variables can be extracted as `&'static str`
- Supports extraction to `std::borrow::Cow` as well.
- No macros (is it good?).

## How to use
You can also see `example` directory.

- In you `Cargo.toml` add the following text  
(your git has to be configured properly to use ssh, and
probably the environment variable `CARGO_NET_GIT_FETCH_WITH_CLI=true` should be set as well)
```toml
[build-dependencies]
i18n = {git = "ssh://git@github.com/Pancir/i18n-gen", version="0.1", package="i18n-gen"}
```
or with a specific commit instead of version
```toml
[build-dependencies]
i18n = {git = "ssh://git@github.com/Pancir/i18n-gen", rev="c098531", package="i18n-gen"}
```
If you have problem in previous step you probably need to read about the git and the rust cargo settings.

- Create a directory where you will store your translations.
This directory must contain file `en-EN.yml` which is considered by the generator as a main template file.  
(You may change default file name via `Config` struct)  
File content example:
```yml
en-EN:
  hello: hello world!
  greet: hello ${name:&str}!
  count: number ${value:u32}!
  group:
    hello: hello world from group!
    greet: hello ${name} from group!
    count: number ${val1:u32} and ${val2:u32} from group!
```
The first line is local code.  
Keys with values are used to generate corresponding functions.  
Keys without values (like `group` in the example) are considered as groups.

Text variables will be translated into functions arguments.  
Syntax of variables:
  - Variables must be quoted with `${}`
  - `${some_text}` variables without a type. The type `&str` will be assigned automatically so,
    `${some_text}` and `${some_text:&str}` are the same.
  - `${val:u32}`  uses the `u32` type.  
  - `${val:&u32}` reference is ok as well.  

For example: the `count: number ${val1:u32} and ${val2:u32} from group!` will be translated into
```text
    tr::group::number(val1: u32, val2: u32) -> some generated return;
```

- Project structure example:
```text
.
├── Cargo.lock
├── Cargo.toml
├── build.rs
├── i18n
│   ├── en-EN.yml
│   ├── ru-RU.yml
└── src
    ├── tr.rs // will be generated
    └── main.rs
```
Actually input/output directories can be chosen.


- In your crate you have to create `build.rs` file with the following code:
```rs
use std::path::PathBuf;

fn main() {
    // A directory where your translation files are located.
    let i18n_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n");

    // A directory where rust module will be generated.
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    
    // Generate rust module code.
    i18n::generate(&i18n_dir, &out_dir, i18n::Config::default()).unwrap();
    
    // Tells cargo to re-run this code whenever 
    // the directory with your translations is changed.
    println!("cargo:rerun-if-changed={}", i18n_dir.display());
}
```

- Usage:
```rs
mod tr;

fn main() {
    // The first you have to set your current local.
    tr::GLOBAL.set_en_en();
    
    // After run building you will be able to use generated code.
    println!("{}", tr::GLOBAL.hello());
    println!("{}", tr::GLOBAL.greet("Man"));
    println!("{}", tr::GLOBAL.count(42));
    println!();

    println!("}", tr::GLOBAL.group.hello());
    println!("}", tr::GLOBAL.group.greet("Man"));
    println!("}", tr::GLOBAL.group.count(42, 52));
    
    // Those functions return structs which implement std::fmt::Display
    // and have some additional useful implementation.
    
    // str() function is available for text without variables and returns &'static str.
    println!("{}", tr::GLOBAL.group.hello().str());
    
    // cow() function is available for returning either 
    // &'static str or String using the std::borrow::Cow.
    println!("{}", tr::GLOBAL.count(42).cow().as_ref());
    
    // Direct access to a local's functions.
    println!("{}", tr::en_en::count(42));
    
    // The current local can be set by its key which is set in the `.yml` file.
    tr::GLOBAL.set("en-EN");
    
    // Ability to get list of existing keys
    let list = tr::list();
    assert_eq!("en-EN", list[0]);
        
    /// Local instance can be used as well.    
    let local = tr::local::Local::new_en_en();
    
    /// Or even a part of local can be used. 
    /// It uses local from the group named "group"
    let local_part = tr::local::group::Local::new_en_en();
}

```
