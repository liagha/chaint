#![allow(dead_code, ambiguous_glob_reexports)]

pub mod formation;
pub mod operation;

pub use formation::*;
pub use operation::*;

pub use formation::Joint as FormationJoint;
pub use operation::Joint as OperationJoint;

use std::sync::atomic::{AtomicUsize, Ordering};

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_identity() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

use std::hash::Hash;
use std::marker::PhantomData;

pub type Identity = usize;
pub type Offset = u32;
pub type Scale = usize;

pub trait Formable<'a>: Clone + Eq + Hash + PartialEq + std::fmt::Display + 'a {}

impl<'a, T> Formable<'a> for T where T: Clone + Eq + Hash + PartialEq + std::fmt::Display + 'a {}

pub trait Combinator<'a, Joint>: Send + Sync {
    fn combinator(&self, joint: &mut Joint);
}

pub struct Multiple<'a, 'source, Joint> {
    pub combinators: Vec<std::sync::Arc<dyn Combinator<'a, Joint> + Send + Sync + 'source>>,
}

pub struct Resolve;

pub struct Depend;

pub struct Pulse {
    pub idle: u64,
}

pub struct Ignore;

pub struct Skip;

pub struct Transform<'bound, Joint, Failure> {
    pub transformer:
        std::sync::Arc<dyn Fn(&mut Joint) -> Result<(), Failure> + Send + Sync + 'bound>,
}

pub struct Fail<'bound, Joint, Failure> {
    pub emitter: std::sync::Arc<dyn Fn(&mut Joint) -> Failure + Send + Sync + 'bound>,
}

pub struct Panic<'bound, Joint, Failure> {
    pub emitter: std::sync::Arc<dyn Fn(&mut Joint) -> Failure + Send + Sync + 'bound>,
}

pub struct Recover<'bound, Joint, Input, Failure> {
    pub sync: std::sync::Arc<dyn Fn(&Input) -> bool + Send + Sync + 'bound>,
    pub emitter: std::sync::Arc<dyn Fn(&mut Joint) -> Failure + Send + Sync + 'bound>,
}

#[derive(Clone)]
pub struct Literal<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub value: std::sync::Arc<dyn PartialEq<Input> + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Predicate<'a, 'source, Input>
where
    Input: Formable<'a>,
{
    pub function: std::sync::Arc<dyn Fn(&Input) -> bool + Send + Sync + 'source>,
    pub phantom: PhantomData<&'a ()>,
}

pub struct Deferred<State> {
    pub factory: fn() -> State,
}

pub struct Optional<State> {
    pub state: Box<State>,
}

pub struct Snapshot<State> {
    pub state: Box<State>,
}

pub struct Group<State> {
    pub state: Box<State>,
}

pub struct Alternative<State, const SIZE: usize> {
    pub states: [State; SIZE],
    pub halt: fn(&State) -> bool,
    pub compare: fn(&State, &State) -> bool,
}

pub struct Sequence<State, const SIZE: usize> {
    pub states: [State; SIZE],
    pub halt: fn(&State) -> bool,
    pub keep: fn(&State) -> bool,
}

pub struct Repetition<State> {
    pub state: Box<State>,
    pub minimum: usize,
    pub maximum: Option<usize>,
    pub halt: fn(&State) -> bool,
    pub keep: fn(&State) -> bool,
}

pub struct Cycle<State> {
    pub state: Box<State>,
    pub keep: fn(&State) -> bool,
}

#[derive(Clone)]
pub struct Memoize<C> {
    pub inner: C,
}

impl<C> Memoize<C> {
    pub fn new(inner: C) -> Self {
        Self { inner }
    }
}

pub trait Peekable<'peekable, Item: PartialEq + 'peekable> {
    type State: Copy + Default + Send + Sync;

    fn length(&self) -> Scale;

    fn remaining(&self) -> Scale {
        self.length() - self.index() as Scale
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Item>;
    fn peek_behind(&self, n: Offset) -> Option<&Item>;

    fn origin(&self) -> Self::State;

    fn reset(&mut self) {
        self.set_index(0);
        self.set_state(self.origin());
    }

    fn advance(&mut self) -> Option<Item> {
        let mut index = self.index();
        let mut state = self.state();
        let result = self.next(&mut index, &mut state);

        if result.is_some() {
            self.set_index(index);
            self.set_state(state);
        }

        result
    }

    fn next(&self, index: &mut Offset, state: &mut Self::State) -> Option<Item>;

    fn get(&self, index: Offset) -> Option<&Item> {
        self.input().get(index as usize)
    }

    fn get_mut(&mut self, index: Offset) -> Option<&mut Item> {
        self.input_mut().get_mut(index as usize)
    }

    fn insert(&mut self, index: Offset, item: Item) {
        self.input_mut().insert(index as usize, item);
    }

    fn remove(&mut self, index: Offset) -> Option<Item> {
        Some(self.input_mut().remove(index as usize))
    }

    fn input(&self) -> &Vec<Item>;
    fn input_mut(&mut self) -> &mut Vec<Item>;
    fn state(&self) -> Self::State;
    fn state_mut(&mut self) -> &mut Self::State;
    fn index(&self) -> Offset;
    fn index_mut(&mut self) -> &mut Offset;

    fn peek(&self) -> Option<&Item> {
        self.peek_ahead(0)
    }

    fn peek_previous(&self) -> Option<&Item> {
        self.peek_behind(1)
    }

    fn set_index(&mut self, index: Offset) {
        *self.index_mut() = index;
    }

    fn set_state(&mut self, state: Self::State) {
        *self.state_mut() = state;
    }

    fn set_input(&mut self, input: Vec<Item>) {
        *self.input_mut() = input;
    }

    fn skip(&mut self, count: Offset) {
        for _ in 0..count {
            self.advance();
        }
    }
}
