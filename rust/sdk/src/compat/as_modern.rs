use super::{backward_compat, AsModern};

#[cfg(feature = "backward-compat")]
impl AsModern for backward_compat::Pubkey {
    type Modern = solana_address::Address;
    fn as_modern(&self) -> &Self::Modern {
        unsafe { &*(self.as_array().as_ptr() as *const Self::Modern) }
    }
}

#[cfg(not(feature = "backward-compat"))]
impl AsModern for backward_compat::Pubkey {
    type Modern = solana_address::Address;
    fn as_modern(&self) -> &Self::Modern {
        self
    }
}

impl<'info> AsModern for backward_compat::AccountInfo<'info> {
    type Modern = solana_program::account_info::AccountInfo<'info>;

    #[cfg(feature = "backward-compat")]
    fn as_modern(&self) -> &Self::Modern {
        const {
            assert!(
                core::mem::size_of::<backward_compat::AccountInfo<'static>>()
                    == core::mem::size_of::<solana_program::account_info::AccountInfo<'static>>()
            );
            assert!(
                core::mem::align_of::<backward_compat::AccountInfo<'static>>()
                    == core::mem::align_of::<solana_program::account_info::AccountInfo<'static>>()
            );
        }

        unsafe { &*(self as *const Self as *const Self::Modern) }
    }

    #[cfg(not(feature = "backward-compat"))]
    fn as_modern(&self) -> &Self::Modern {
        self
    }
}

impl AsModern for () {
    type Modern = ();
    fn as_modern(&self) -> &Self::Modern {
        self
    }
}
