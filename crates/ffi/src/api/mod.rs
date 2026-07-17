//! Bridge API surface for the Flutter app.
//!
//! Hello-world placeholder to prove the FRB pipeline; the real surface
//! delegates to `koi-app` per spec/MOBILE.md.

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

#[flutter_rust_bridge::frb(sync)]
pub fn greet(name: String) -> String {
    format!("Hello, {name}! — from the koi rust core")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_greets() {
        assert_eq!(greet("koi".into()), "Hello, koi! — from the koi rust core");
    }
}
