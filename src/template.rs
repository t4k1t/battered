pub trait Template {
    fn placeholders(&self) -> Vec<(String, String)>;
}

// macro to implement the Template trait for a struct
macro_rules! impl_template {
    ($struct_name:ident { $($field_name:ident),* }) => {
        impl Template for $struct_name<'_> {
            fn placeholders(&self) -> Vec<(String, String)> {
                vec![$(
                    (stringify!($field_name).to_string(), self.$field_name.to_string())
                ),*]
            }
        }
    };
}

pub struct FormatObject<'f> {
    pub percentage: &'f f32,
    pub state: &'f str,
    pub energy_rate: &'f f32,
}

impl_template!(FormatObject {
    percentage,
    state,
    energy_rate
});
