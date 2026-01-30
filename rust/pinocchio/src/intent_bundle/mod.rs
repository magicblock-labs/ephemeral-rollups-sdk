use pinocchio::AccountView;

mod args;


// TODO: rename CallHandler -> BaseActions


pub struct MagicInstructionBuilder<'a1, 'a2, 'act1, 'act2, 'act3, 'act4> {
    payer: AccountView,
    magic_context: AccountView,
    magic_program: AccountView,
    magic_intent_bundle: MagicIntentBundleBuilder<'a1, 'a2, 'act1, 'act2, 'act3, 'act4>,
}

impl<'a1, 'a2, 'act1, 'act2, 'act3, 'act4> MagicIntentBundleBuilder<'a1, 'a2, 'act1, 'act2, 'act3, 'act4> {
    pub fn new(payer: AccountView, magic_context: AccountView, magic_program: AccountView)
}

pub struct MagicIntentBundle<'a1, 'a2, 'act1, 'act2, 'act3, 'act4> {
    standalone_actions: &'act1 [CallHandler],
    commit_intent: Option<CommitIntent<'a1, 'act2>>,
    commit_and_undelegate_intent: Option<CommitAndUndelegateIntent<'a2, 'act3, 'act4>>
}

impl Default for MagicIntentBundle<'static, 'static, 'static, 'static, 'static, 'static> {
    fn default() -> Self {
        Self {
            standalone_actions: &[],
            commit_intent: None,
            commit_and_undelegate_intent: None
        }
    }
}


pub struct CommitIntent<'a, 'act> {
    accounts: &'a [AccountView],
    post_commit_actions: &'act [CallHandler]
}

pub struct CommitAndUndelegateIntent<'a, 'act1, 'act2> {
    accounts: &'a [AccountView],
    post_commit_actions: &'act1 [CallHandler]
}

pub struct CommitIntentBuilder<'a, 'act, 'pa1, 'pa2, 'pact1, 'pact2, 'pact3, 'pact4> {
    parent: MagicIntentBundleBuilder<'pa1, 'pa2, 'pact1, 'pact2, 'pact3, 'pact4>,
    accounts: &'a [AccountView],
    actions: &'act [CallHandler]
}


impl<'a, 'act, 'pa1, 'pa2, 'pact1, 'pact2, 'pact3, 'pact4> CommitIntentBuilder<'a, 'act, 'pa1, 'pa2, 'pact1, 'pact2, 'pact3, 'pact4> {
    pub fn add_post_commit_actions<'act2>(
        mut self,
        actions: &'act2 [CallHandler],
    ) -> CommitIntentBuilder<'a, 'act2> {
        CommitIntentBuilder {
            accounts: self.accounts,
            actions
        }
    }

    fn done(self) -> MagicIntentBundleBuilder {
        let intent = CommitIntent {
            accounts: self.accounts,
            post_commit_actions: self.actions
        };


        todo!()
    }
}

impl<'a, 'act> CommitType<'a, 'act> {
    pub fn new(committed_accounts: &'a [AccountView]) -> Self {
        Self {
            committed_accounts,
            call_handlers: &[]
        }
    }
}

fn test(accounts: &[AccountView]) {
    let asd = CommitType::new(accounts);
    let jej = asd.call_handlers;
}