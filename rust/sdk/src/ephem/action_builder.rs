use crate::ephem::cau_intent_builder::{
    CommitAndUndelegateIntentBuilder, FoldableCauIntentBuilder,
};
use crate::ephem::commit_intent_builder::{CommitIntentBuilder, FoldableCommitIntentBuilder};
use crate::ephem::{ActionCallback, CallHandler, FoldableIntentBuilder, MagicIntentBundleBuilder};

pub struct ActionBuilder<'info, T, F> {
    parent: T,
    action: CallHandler<'info>,
    callback: Option<ActionCallback>,
    f: F,
}

impl<'info, T, F> ActionBuilder<'info, T, F>
where
    F: FnOnce(T, CallHandler<'info>, Option<ActionCallback>) -> T,
{
    pub fn new(parent: T, action: CallHandler<'info>, f: F) -> Self {
        Self {
            parent,
            action,
            callback: None,
            f,
        }
    }

    pub fn then(mut self, callback: ActionCallback) -> Self {
        self.callback = Some(callback);
        self
    }
}

impl<'info, F> FoldableCommitIntentBuilder<'info>
    for ActionBuilder<'info, CommitIntentBuilder<'info>, F>
where
    F: FnOnce(
        CommitIntentBuilder<'info>,
        CallHandler<'info>,
        Option<ActionCallback>,
    ) -> CommitIntentBuilder<'info>,
{
    fn fold_commit_builder(self) -> CommitIntentBuilder<'info> {
        (self.f)(self.parent, self.action, self.callback)
    }
}

impl<'info, F> FoldableIntentBuilder<'info> for ActionBuilder<'info, CommitIntentBuilder<'info>, F>
where
    F: FnOnce(
        CommitIntentBuilder<'info>,
        CallHandler<'info>,
        Option<ActionCallback>,
    ) -> CommitIntentBuilder<'info>,
{
    fn fold_builder(self) -> MagicIntentBundleBuilder<'info> {
        (self.f)(self.parent, self.action, self.callback).fold_builder()
    }
}

impl<'info, F> FoldableCauIntentBuilder<'info>
    for ActionBuilder<'info, CommitAndUndelegateIntentBuilder<'info>, F>
where
    F: FnOnce(
        CommitAndUndelegateIntentBuilder<'info>,
        CallHandler<'info>,
        Option<ActionCallback>,
    ) -> CommitAndUndelegateIntentBuilder<'info>,
{
    fn fold_cau_builder(self) -> CommitAndUndelegateIntentBuilder<'info> {
        (self.f)(self.parent, self.action, self.callback)
    }
}

impl<'info, F> FoldableIntentBuilder<'info>
    for ActionBuilder<'info, CommitAndUndelegateIntentBuilder<'info>, F>
where
    F: FnOnce(
        CommitAndUndelegateIntentBuilder<'info>,
        CallHandler<'info>,
        Option<ActionCallback>,
    ) -> CommitAndUndelegateIntentBuilder<'info>,
{
    fn fold_builder(self) -> MagicIntentBundleBuilder<'info> {
        (self.f)(self.parent, self.action, self.callback).fold_builder()
    }
}

impl<'info, F> FoldableIntentBuilder<'info>
    for ActionBuilder<'info, MagicIntentBundleBuilder<'info>, F>
where
    F: FnOnce(
        MagicIntentBundleBuilder<'info>,
        CallHandler<'info>,
        Option<ActionCallback>,
    ) -> MagicIntentBundleBuilder<'info>,
{
    fn fold_builder(self) -> MagicIntentBundleBuilder<'info> {
        (self.f)(self.parent, self.action, self.callback)
    }
}
