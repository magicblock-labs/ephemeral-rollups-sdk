use crate::ephem::deprecated::v1::MagicAction;
use crate::solana_compat::solana::{AccountInfo, Pubkey};

pub use crate::ephem::deprecated::v0::{
    commit_accounts, commit_and_undelegate_accounts, create_schedule_commit_ix,
};

pub mod deprecated;

/// The user can really do:
/// 1. Commit account
/// 2. Commit & undelegate account
/// 3. Run action
/// There's no such thing as just Undelegation
/// There's also no separate instruction on finalize
/// 1. Should we allow separate Finalize ix?
///     No, since this is a validators business. User's can't finalize an account
/// 2. Should we allow separate undelegate ix?
///     That could resolve a case of unsuccessful undelegation
///     BUT, its not clear if account failed or not, unless this info is available
///     We can't guarantee Intent execution
///     is_delegated = false, is_awaiting = true
///
///     The issue with above is that we can't tell if it failed or was just scheduled
///     and we didn't have enough time to undelegate. RACE-CONDITION
///
///     So while would be cool - `No`, unless new flag - failed_to_undelegate :-)

/// Describes types of Base Intents
pub enum BaseIntent<'info> {
    CommitIntent(AccountInfo<'info>),
    CommitAndUndelegateIntent(AccountInfo<'info>),
    ActionIntent(MagicAction<'info>),
}

pub struct MagicIntentsBuilder<'info> {
    payer: AccountInfo<'info>,
    magic_context: AccountInfo<'info>,
    magic_program: AccountInfo<'info>,
    intents: Vec<BaseIntent<'info>>,
}

impl<'info> MagicIntentsBuilder<'info> {
    pub fn new(
        payer: AccountInfo<'info>,
        magic_context: AccountInfo<'info>,
        magic_program: AccountInfo<'info>,
    ) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            intents: vec![],
        }
    }

    pub fn commit(self, account: AccountInfo<'info>) -> CommitIntentBuilder {
        CommitIntentBuilder {
            builder: self,
            account,
            undelegate: false,
        }
    }

    pub fn commit_multiple<'a>(
        self,
        accounts: &'a [AccountInfo<'info>],
    ) -> CommitMultipleBuilder<'a, 'info> {
        CommitMultipleBuilder {
            builder: self,
            accounts,
            undelegate: false,
        }
    }
}

pub struct CommitIntentBuilder<'info> {
    builder: MagicIntentsBuilder<'info>,
    account: AccountInfo<'info>,
    undelegate: bool,
}

impl<'info> CommitIntentBuilder<'info> {
    pub fn undelegate(mut self) -> MagicIntentsBuilder<'info> {
        self.undelegate = true;
        self.consume()
    }

    pub fn commit(mut self, account: AccountInfo<'info>) -> CommitIntentBuilder<'info> {
        let builder = self.consume();
        Self {
            builder,
            account,
            undelegate: false,
        }
    }

    pub fn commit_multiple<'a>(
        self,
        accounts: &'a [AccountInfo<'info>],
    ) -> CommitMultipleBuilder<'a, 'info> {
        let builder = self.consume();
        CommitMultipleBuilder {
            builder,
            accounts,
            undelegate: false,
        }
    }

    /// Consume current builder
    /// Build Commit Intent Type and add it to `MagicIntentsBuilder`
    fn consume(mut self) -> MagicIntentsBuilder<'info> {
        let intent = if self.undelegate {
            BaseIntent::CommitIntent(self.account)
        } else {
            BaseIntent::CommitAndUndelegateIntent(self.account)
        };
        self.builder.intents.push(intent);

        self.builder
    }
}

pub struct CommitMultipleBuilder<'a, 'info> {
    builder: MagicIntentsBuilder<'info>,
    accounts: &'a [AccountInfo<'info>],
    undelegate: bool,
}

impl<'a, 'info> CommitMultipleBuilder<'a, 'info> {
    fn new(builder: MagicIntentsBuilder<'info>, accounts: &'a [AccountInfo<'info>]) -> Self {
        Self {
            builder,
            accounts,
            undelegate: false,
        }
    }

    pub fn undelegate(mut self) -> MagicIntentsBuilder<'info> {
        self.undelegate = true;
        self.consume()
    }

    pub fn commit(self, account: AccountInfo<'info>) -> CommitIntentBuilder<'info> {
        let builder = self.consume();
        CommitIntentBuilder {
            builder,
            account,
            undelegate: false,
        }
    }

    pub fn consume(mut self) -> MagicIntentsBuilder<'info> {
        let iter = self.accounts.into_iter().map(|account| {
            if self.undelegate {
                BaseIntent::CommitAndUndelegateIntent(account.clone())
            } else {
                BaseIntent::CommitIntent(account.clone())
            }
        });
        self.builder.intents.extend(iter);

        self.builder
    }
}

fn asd<'info>(accounts: &[AccountInfo<'info>]) {
    let ([payer, magic_context, magic_program, pda1, pda2, pda3], others) = accounts.split_at(3);
    MagicIntentsBuilder::new(payer.clone(), magic_context.clone(), magic_program.clone())
        .commit(pda1.clone())
        .undelegate()
        .commit_multiple(others)
        .undelegate()
        .add_action(action1)
        .commit(pda2.clone())
        .add_action(action2)
        .build_and_invoke();

    todo!()
}
