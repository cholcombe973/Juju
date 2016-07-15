///
/// A basic macro to ease creation and management
/// of Hooks
///
#[macro_export]
macro_rules! hook {
    ($hook_name:expr, $fn_name:ident) => {
        $crate::Hook {
            name: $hook_name.to_string(),
            callback: $fn_name,
        }
    };
    ($hook_name:ident $fn_name:ident) => {
        $crate::Hook {
            name: stringify!($hook_name).to_string(),
            callback: $fn_name,
        }
    };
}

#[macro_export]
macro_rules! status_set {
    ($status_type:ident $message:expr) => {
        $crate::status_set(
            $crate::Status {
                status_type: $crate::StatusType::$status_type,
                message: $message.to_string()
            }
        )
    }
}

#[cfg(test)]
mod tests{
    #[allow(dead_code)]
    mod status_set {
        fn it_compiles_correctly() {
            let _ = status_set!(Maintenance "Doing stuff");
        }
    }
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

    #[test]
    fn it_makes_a_complex_named_hook_correctly() {
        let h1 = hook!("config-changed", cb);
        let h2 = Hook {
            name: "config-changed".to_string(),
            callback: cb,
        };
        assert_eq!(h1, h2);
    }
}