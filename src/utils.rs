macro_rules! impl_len {
    ($struct_name:ident) => {
        impl $struct_name {
            pub const LEN: usize = core::mem::size_of::<$struct_name>();
        }
    };
}

macro_rules! impl_load {
    ($struct_name:ident) => {
        impl $struct_name {
            pub fn load(data: &[u8]) -> Result<&Self, pinocchio::error::ProgramError> {
                if data.len() != Self::LEN {
                    return Err(pinocchio::error::ProgramError::InvalidAccountData);
                }
                // it is safe to transmute here because we have already checked the length and the struct is `Pod`
                Ok(bytemuck::from_bytes(data))
            }

            pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, pinocchio::error::ProgramError> {
                if data.len() != Self::LEN {
                    return Err(pinocchio::error::ProgramError::InvalidAccountData);
                }
                Ok(bytemuck::from_bytes_mut(data))
            }
        }
    };
}   

pub(crate) use impl_len;
pub(crate) use impl_load;
