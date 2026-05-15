use super::AsModern;

#[cfg(feature = "backward-compat")]
impl AsModern for super::backward_compat::Pubkey {
    type Modern = solana_address::Address;
    fn as_modern(&self) -> &Self::Modern {
        unsafe { &*(self.as_array().as_ptr() as *const Self::Modern) }
    }
}

impl AsModern for super::latest::Pubkey {
    type Modern = solana_address::Address;
    fn as_modern(&self) -> &Self::Modern {
        self
    }
}

#[cfg(feature = "backward-compat")]
impl<'info> AsModern for super::backward_compat::AccountInfo<'info> {
    type Modern = solana_program::account_info::AccountInfo<'info>;

    fn as_modern(&self) -> &Self::Modern {
        assert!(
            core::mem::size_of::<super::backward_compat::AccountInfo<'static>>()
                == core::mem::size_of::<solana_program::account_info::AccountInfo<'static>>()
        );
        assert!(
            core::mem::align_of::<super::backward_compat::AccountInfo<'static>>()
                == core::mem::align_of::<solana_program::account_info::AccountInfo<'static>>()
        );
        unsafe { &*(self as *const Self as *const Self::Modern) }
    }
}

impl<'info> AsModern for super::latest::AccountInfo<'info> {
    type Modern = solana_program::account_info::AccountInfo<'info>;

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
