//! Common traits and utilities to reduce code duplication

use std::collections::HashMap;

/// Trait for types that can check for key existence
pub trait HasKey<K: ?Sized> {
    fn has_key(&self, key: &K) -> bool;
}

/// Trait for types that can validate their configuration
pub trait Validate {
    type Error;
    
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Trait for types that provide statistics
pub trait Stats {
    type StatsType;
    
    fn stats(&self) -> &Self::StatsType;
}

/// Trait for types that can be cleared/reset
pub trait Clear {
    fn clear(&mut self);
}

/// Macro to generate Default implementations with common patterns
macro_rules! impl_default_config {
    ($type:ty, {
        $($field:ident: $value:expr),* $(,)?
    }) => {
        impl Default for $type {
            fn default() -> Self {
                Self {
                    $($field: $value,)*
                }
            }
        }
    };
}

/// Macro to generate validation methods with common patterns
macro_rules! impl_validation {
    ($type:ty, $error:ty, {
        $($field:ident: $validation:expr => $error_msg:expr),* $(,)?
    }) => {
        impl Validate for $type {
            type Error = $error;
            
            fn validate(&self) -> Result<(), Self::Error> {
                $(
                    if !($validation(&self.$field)) {
                        return Err(<$error>::ValidationError($error_msg.to_string()));
                    }
                )*
                Ok(())
            }
        }
    };
}

// Export macros
pub(crate) use impl_default_config;
pub(crate) use impl_validation;

// Common validation functions
pub(crate) fn is_positive_usize(value: &usize) -> bool {
    *value > 0
}

pub(crate) fn is_valid_buffer_size(value: &usize) -> bool {
    *value >= 1024 && *value <= 1024 * 1024 // Between 1KB and 1MB
}

pub(crate) fn is_reasonable_depth(value: &usize) -> bool {
    *value > 0 && *value <= 1000
}

// Implementation for HashMap<String, V> to accept &str
impl<V> HasKey<str> for HashMap<String, V> {
    fn has_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }
}

// Generic implementation for HashMap-based collections
impl<K, V> HasKey<K> for HashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn has_key(&self, key: &K) -> bool {
        self.contains_key(key)
    }
}