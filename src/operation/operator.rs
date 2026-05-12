use crate::{Depend, Identity, Operation, Pulse, Resolve, Status};
use std::collections::HashMap;
use std::time::Duration;

struct Cache;

impl Cache {
    #[inline]
    pub fn get(cache: &HashMap<Identity, Status>, identity: Identity) -> Option<Status> {
        cache.get(&identity).cloned()
    }

    #[inline]
    pub fn put(cache: &mut HashMap<Identity, Status>, identity: Identity, status: Status) {
        cache.insert(identity, status);
    }

    #[inline]
    pub fn reset(cache: &mut HashMap<Identity, Status>) {
        cache.clear();
    }
}

impl Resolve {
    #[inline]
    pub fn run<'op, 'source, Store: Clone + Send + Sync>(
        operator: &'op mut Operator<Store>,
        operation: &'op mut Operation<'source, Store>,
    ) {
        let combinator = operation.combinator.clone();
        let mut joint = (operator, operation);
        combinator.combinator(&mut joint);
    }
}

impl Depend {
    #[inline]
    pub fn run<'op, 'source, Store: Clone + Send + Sync>(
        operator: &'op mut Operator<Store>,
        operation: &'op mut Operation<'source, Store>,
    ) -> bool {
        for dependency in &operation.depends {
            if let Some(status) = operator.cache.get(dependency) {
                if !matches!(status, Status::Resolved(_)) {
                    operation.set_reject();
                    return false;
                }
            } else {
                operation.set_pending();
                return false;
            }
        }
        true
    }
}

impl Pulse {
    #[inline]
    pub fn tick(&self) {
        std::thread::sleep(Duration::from_millis(self.idle));
    }
}

pub struct Operator<Store = ()> {
    pub cache: HashMap<Identity, Status>,
    pub store: Store,
}

impl<Store: Clone + Send + Sync> Operator<Store> {
    #[inline]
    pub fn new(store: Store) -> Self {
        Self {
            cache: HashMap::new(),
            store,
        }
    }

    #[inline]
    pub fn build<'op, 'source>(&'op mut self, operation: &'op mut Operation<'source, Store>) {
        if let Some(status) = self.cache.get(&operation.identity).cloned() {
            operation.status = status;
            return;
        }

        if !Depend::run(self, operation) {
            return;
        }

        Resolve::run(self, operation);

        if !operation.is_pending() {
            self.cache
                .insert(operation.identity, operation.status.clone());
        }
    }

    #[inline]
    pub fn execute<'op, 'source>(
        &'op mut self,
        operation: &'op mut Operation<'source, Store>,
    ) -> Status {
        loop {
            self.build(operation);

            match &operation.status {
                Status::Pending => {
                    Pulse { idle: 10 }.tick();
                }
                Status::Resolved(_) | Status::Rejected => break operation.status.clone(),
            }
        }
    }

    #[inline]
    pub fn watch<'op, 'source>(
        &'op mut self,
        operation: &'op mut Operation<'source, Store>,
        paths: &[&str],
    ) {
        let mut last: Vec<_> = paths
            .iter()
            .map(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok())
            .collect();

        loop {
            Cache::reset(&mut self.cache);
            operation.status = Status::Pending;
            self.execute(operation);

            loop {
                Pulse { idle: 500 }.tick();

                let current: Vec<_> = paths
                    .iter()
                    .map(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok())
                    .collect();

                if current != last {
                    last = current;
                    break;
                }
            }
        }
    }
}
