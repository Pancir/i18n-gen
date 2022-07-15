# Library for generating internationalization (i18n) rust module files.

## WARNING
- The library is in a **prototype** state. 
- It does not have a stable interface so, any new version may break backward compatibility.   
- Also, it is not presented in the [crates.io](https://crates.io/) at this moment.  
- As it is a prototype it does not have a good internal architecture yet.  
- There is `unsafe` code at this moment. (used for `static mut` variable which represents current selected local)  
- It supports only text translations. It does not support formatting for dates, currency, values etc...
- It does not support runtime loading the translation files.


## Features
- Generating code for including into your binary. Actually it generates rust module (code).
- `.yml` files for storing translations.
- It is a dependency for compile(build) time only. Your code needs to `use` only generated file which uses only standard library.   
(Can be changed in the future)
- Allocation free. Generated functions return structures which implement `std::fmt::Display` instead of `String` so, 
you are responsible what to do in the next step. There is no new String creation (so - no allocation).
- Translation text variables support types. (see examples)
- Any types including custom ones which implement `std::fmt::Display` can be used as a text variable.
- Text without variables can be extracted as `&'static str`
- Extraction to `std::borrow::Cow` as well.
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
or with certain commit instead of version
```toml
[build-dependencies]
i18n = {git = "ssh://git@github.com/Pancir/i18n-gen", rev="2babfa1", package="i18n-gen"}
```
If you have problem in that step you probably need to read about git and cargo settings.

- Create a directory where you will store your translations.
This directory must contain file `!defatul.yml` which is considered as a main template file by the generator.  
Example:
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
*WARN:* Only one group level is allowed at this moment.  

Text variable will be translated into functions arguments.  
Syntax of variables:
  - Variables must be quoted with `${}`
  - `${some_text}` variables without a type. The type `&str` will be assigned automatically.
  - `${some_text}` and `${some_text:&str}` are the same.
  - `${val:u32}` example of type `u32`.  
  - `${val:&u32}` reference is ok as well.  

Example: `count: number ${val1:u32} and ${val2:u32} from group!` will be translated into
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
│   ├── !default.yml // ! - is used for sorting (this file is always on top)
│   ├── ru-RU.yml
└── src
    ├── tr.rs // generated
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
    i18n::generate(&i18n_dir, &out_dir).unwrap();
    
    // Tells cargo to re-run this code whenever 
    // the directory with your translations is changed.
    println!("cargo:rerun-if-changed={}", i18n_dir.display());
}
```

- Usage (somewhere in your code):
```rs
mod tr;

fn main() {
    // Firstly you have to set your current local if it should not be a default one.
    // It is unsafe at this moment because internally 
    // it need to access to a mut static internal variable.
    // So this function IS NOT THREAD SAFE!
    // I will consider to remove unsafe code in future.
    unsafe { tr::service::set_en_en() };
    
    // After run building you will be able to use generated code.
    println!("Default local");
    println!("  {}", tr::hello());
    println!("  {}", tr::greet("Man"));
    println!("  {}", tr::count(42));
    println!();

    println!("Default local: Group usage");
    println!("  {}", tr::group::hello());
    println!("  {}", tr::group::greet("Man"));
    println!("  {}", tr::group::count(42, 52));
    
    // Those functions return structs which implement std::fmt::Display
    // and have some additional useful implementation.
    
    // str() function is available for text without variables and returns &'static str.
    println!("  {}", tr::group::hello().str());
    
    // cow() function is available for returning either 
    // &'static str or String using the std::borrow::Cow.
    println!("  {}", tr::count(42).cow().as_ref());
    
    // Direct access to a local's functions.
    println!("  {}", tr::en_en::count(42));
}

```
