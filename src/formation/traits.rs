use crate::{Formable, Peekable, formation::formation::Formation};

impl<'a, 'source, Source, Input, Output, Failure> Clone
for Formation<'a, 'source, Source, Input, Output, Failure>
where
    Source: Peekable<'a, Input> + Clone,
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    fn clone(&self) -> Self {
        Self {
            identity: self.identity,
            combinator: self.combinator.clone(),
            marker: self.marker,
            state: self.state,
            consumed: self.consumed.clone(),
            outcome: self.outcome.clone(),
            form: self.form.clone(),
            stack: self.stack.clone(),
            depth: self.depth,
        }
    }
}