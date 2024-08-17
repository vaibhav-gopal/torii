# Project Structure

## Error Handling (for this lib crate)
> Currently following [this article](https://sabrinajewson.org/blog/errors#problem-2-inextensibility) and [this article](https://www.lpalmieri.com/posts/error-handling-rust/#the-error-trait)

* Do not use `.unwrap()`, `.expect()`, etc in Library crates --> propagate the error --> let application handle it
  * Using `.unwrap_or()` or default value type functions are ok! (Nothing that panics!)
* Within each error-prone module include an `Error` and `Result<T>` type (or expose it from a submodule named `error`)
  * `my_module::Error` and `my_module::Result<T>`
  * The `my_module::Result<T>` type needs to wrap the `Result<T, E>` using `my_module::Error`
  * The `my_module::Result<T>` is a type alias
    * `pub type Result<T> = std::result::Result<T, Error>`
  * The `my_module::Error` type is implemented using `thiserror` crate
    * Use the `#[derive(Error, Debug)]` to derive `Error` trait on the new `Error` type
    * Use the `#[error(fmt_msg)]` from `thiserror` to mark error messages on errors (`Error` type fields) (automatically impl/derive `Display`)
      * Use the `#[error(transparent)]` tag, to forward the source and Display methods to the underlying error marked with `#[from]` or `#[source]`
    * Use the `#[from]` to mark fields as errors from other modules (external errors)
      * errors can only have one field, and that must implement `From`
      * Example `IoError(#[from] io::Error)`
    * Chain errors from other modules using `#[source]` by marking a field or naming the field `source`
      * For example an error like: `PhysicsSubsystemError {msg: String, source: physics_module::Error}`
      * Or: `PhysicsSubsystemError {msg: String, #[source] submodule: physics_module::Error}`
    * Both `#[from]` and `#[source]` both implement the source field, `#[from]` is a subset of `#[source]`
    * Use enum structs rather than enum tuples to hold error information (named information)
      * Ideally, implement `#[from]` and `#[error(fmt_msg)]` on every error to impl `Display` and `From` traits
    * For early development you can forego implementing a custom `pub enum Error` type, and just use a type alias for any error:
      * `pub type Error = Box<dyn std::error::Error>`
* Propagate errors using `(...)?` syntax
* Pattern is similar to `anyhow` crate, but we redefine specific Error types for each module rather than encapsulating it away
* Error-prone functions should always return `Result<T>`
  * Non-error-prone functions (guaranteed to not raise an error) shouldn't return Result
  * Error-prone functions that don't return a value should still return `Result<()>` instead