#[macro_export]
macro_rules! impl_atomic_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub fn $getter_name(&self) -> $t {
            self.$field.load(std::sync::atomic::Ordering::Relaxed)
        }

        pub fn $setter_name(&self, new_val: $t) -> () {
            self.$field.store(new_val, std::sync::atomic::Ordering::Relaxed);
        }
    };
}


#[macro_export]
macro_rules! impl_mutex_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub fn $getter_name(&self) -> $t {
            *self.$field.lock().unwrap()
        }

        pub fn $setter_name(&self, new_val: $t) -> () {
            let mut $field: std::sync::MutexGuard<'_, $t> = self.$field.lock().unwrap();
            *$field = new_val;
        }
    };
}


#[macro_export]
macro_rules! impl_tokio_mutex_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub async fn $getter_name(&self) -> $t {
            *self.$field.lock().unwrap()
        }

        pub async fn $setter_name(&self, new_val: $t) -> () {
            let mut $field: std::sync::MutexGuard<'_, $t> = self.$field.lock().unwrap();
            *$field = new_val;
        }
    };
}


#[macro_export]
macro_rules! impl_rwlock_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub fn $getter_name(&self) -> std::sync::RwLockReadGuard<'_, $t> {
            self.$field.read().unwrap()
        }

        pub fn $setter_name(&self, new_val: $t) -> () {
            let mut $field: std::sync::RwLockWriteGuard<'_, $t> = self.$field.write().unwrap();
            *$field = new_val;
        }
    };
}


#[macro_export]
macro_rules! impl_tokio_rwlock_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub async fn $getter_name(&self) -> tokio::sync::RwLockReadGuard<'_, $t> {
            self.$field.read().await
        }

        pub async fn $setter_name(&self, new_val: $t) -> () {
            let mut $field: tokio::sync::RwLockWriteGuard<'_, $t> = self.$field.write().await;
            *$field = new_val;
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


#[macro_export]
macro_rules! impl_option_access {
    ($field:ident, $t:ty, $getter_name:ident, $setter_name:ident) => {
        pub fn $getter_name(&self) -> Option<$t> {
            self.$field
        }
        
        pub fn $setter_name(&self, new_val: $t) -> () {
            let mut $field: Option<$t> = self.$field;
            $field = Some(new_val);
        }
    };
}