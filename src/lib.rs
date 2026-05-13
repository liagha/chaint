// src/lib.rs

#![allow(dead_code, ambiguous_glob_reexports)]

pub mod formation;
pub mod operation;
mod peek;

pub use formation::*;
pub use operation::*;
pub use peek::{Peekable, Peeker};

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

pub trait Formable<'a>: Clone + Eq + Hash + PartialEq + 'a {}

impl<'a, T> Formable<'a> for T where T: Clone + Eq + Hash + PartialEq + 'a {}

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
