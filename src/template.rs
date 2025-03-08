use std::fmt::Write;

pub trait Template {
    fn to_template(&self) -> String;
}

// macro to implement the Template trait for a struct
macro_rules! impl_template {
    ($struct_name:ident { $($field_name:ident),* }) => {
        impl Template for $struct_name<'_> {
            // Transform fields into '<field>: <value>'-format string
            fn to_template(&self) -> String {
                let mut result = String::new();
                $(
                    let _ = write!(result, "{}: {}\n", stringify!($field_name), &self.$field_name);
                )*
                result
            }
        }
    };
}

pub struct FormatObject<'f> {
    pub percentage: &'f f32,
}

impl_template!(FormatObject { percentage });
