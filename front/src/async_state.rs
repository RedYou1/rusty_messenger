use std::sync::{Arc, Mutex};

use dioxus::prelude::{use_coroutine, Coroutine, Scoped, UseSharedState};
use futures_channel::mpsc::UnboundedReceiver;
use futures_lite::StreamExt;

pub struct AsyncStateSetter<Value>(Arc<Mutex<Coroutine<Value>>>);

impl<Value> AsyncStateSetter<Value>
where
    Value: 'static,
{
    pub fn new<T, Container, Func>(
        cx: &Scoped<'_, T>,
        state: &UseSharedState<Container>,
        func: Func,
    ) -> AsyncStateSetter<Value>
    where
        Container: 'static,
        Func: Fn(&UseSharedState<Container>, Value) + 'static,
    {
        let state = state.to_owned();
        AsyncStateSetter::<Value> {
            0: Arc::new(Mutex::new(
                use_coroutine(cx, |mut receiver: UnboundedReceiver<Value>| async move {
                    loop {
                        match receiver.next().await {
                            Some(v) => func(&state, v),
                            None => panic!("AsyncStateSetter receive None"),
                        }
                    }
                })
                .to_owned(),
            )),
        }
    }

    pub fn set_state(&self, value: Value) {
        let s = self.0.as_ref().lock().unwrap();
        s.send(value);
    }
}

impl<Value> Clone for AsyncStateSetter<Value>
where
    Value: 'static,
{
    fn clone(&self) -> Self {
        AsyncStateSetter::<Value> { 0: self.0.clone() }
    }
}
