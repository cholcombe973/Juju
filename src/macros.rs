///
/// A basic macro to ease creation and management
/// of Hooks
///
#[macro_export]
macro_rules! hook {
    ($hook_name:ident $fn_name:ident) => {
        $crate::Hook {
            name: stringify!($hook_name).to_string(),
            callback: $fn_name,
        }
    }
}

#[cfg(test)]
mod tests{
    use super::super::Hook;
    fn cb() -> Result<(),String> {
        Ok(())
    }
    #[test]
    fn it_makes_a_hook_correctly() {
        let h1 = hook!(test cb);
        let h2 = Hook {
            name: "test".to_string(),
            callback: cb,
        };
        assert_eq!(h1, h2);
    }
}