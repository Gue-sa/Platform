#[macro_export]
macro_rules! impl_atomic_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub fn $getter_name(&self) -> $t {
            self.$field.load(std::sync::atomic::Ordering::Relaxed)
        }

        pub fn $setter_name(&self, new_val: $t) -> () {
            self.$field
                .store(new_val, std::sync::atomic::Ordering::Relaxed);
        }
    };
}

#[macro_export]
macro_rules! impl_arc_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub fn $getter_name(&self) -> $t {
            std::sync::Arc::clone(&self.$field)
        }

        pub fn $setter_name(&self, new_val: $t) -> () {
            let mut $field: $t = std::sync::Arc::clone(&new_val);
        }
    };
}