use deno_core::{
    v8::{self, PromiseState},
    PollEventLoopOptions,
};
use serde::Deserialize;

use super::V8Value;
use crate::{async_bridge::AsyncBridgeExt, Error};

/// A Deserializable javascript promise, that can be stored and used later
/// Must live as long as the runtime it was birthed from
///
/// You can turn `Promise<T>` into `Future<Output = T>` by calling `Promise::into_future`
/// This allows you to export multiple concurrent promises without borrowing the runtime mutably
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Promise<T>(V8Value<PromiseTypeChecker>, std::marker::PhantomData<T>)
where
    T: serde::de::DeserializeOwned;
impl_v8!(Promise<T>, PromiseTypeChecker);
impl_checker!(PromiseTypeChecker, Promise, is_promise, |e| {
    crate::Error::JsonDecode(format!("Expected a promise, found `{e}`"))
});

impl<T> Promise<T>
where
    T: serde::de::DeserializeOwned,
{
    pub(crate) async fn resolve(
        self,
        runtime: &mut deno_core::JsRuntime,
    ) -> Result<T, crate::Error> {
        let future = runtime.resolve(self.0 .0);
        let result = runtime
            .with_event_loop_future(future, PollEventLoopOptions::default())
            .await?;
        deno_core::scope!(scope, runtime);
        let local = v8::Local::new(scope, &result);
        Ok(deno_core::serde_v8::from_v8(scope, local)?)
    }

    /// Returns a future that resolves the promise
    ///
    /// # Errors
    /// Will return an error if the promise cannot be resolved into the given type,
    /// or if a runtime error occurs
    pub async fn into_future(self, runtime: &mut crate::Runtime) -> Result<T, crate::Error> {
        self.resolve(runtime.deno_runtime()).await
    }

    /// Blocks until the promise is resolved
    ///
    /// # Errors
    /// Will return an error if the promise cannot be resolved into the given type,
    /// or if a runtime error occurs
    pub fn into_value(self, runtime: &mut crate::Runtime) -> Result<T, crate::Error> {
        runtime.block_on(move |runtime| async move { self.into_future(runtime).await })
    }

    /// Checks if the promise is pending or already resolved
    pub fn is_pending(&self, runtime: &mut crate::Runtime) -> bool {
        let rt = runtime.deno_runtime();
        deno_core::scope!(scope, rt);
        let value = self.0.as_local(scope);
        value.state() == v8::PromiseState::Pending
    }

    /// Polls the promise, returning `Poll::Pending` if the promise is still pending
    /// or `Poll::Ready(Ok(T))` if the promise is resolved
    /// or `Poll::Ready(Err(Error))` if the promise is rejected
    pub fn poll_promise(&self, runtime: &mut crate::Runtime) -> std::task::Poll<Result<T, Error>> {
        let rt = runtime.deno_runtime();
        deno_core::scope!(scope, rt);
        let value = self.0.as_local(scope);

        match value.state() {
            PromiseState::Pending => std::task::Poll::Pending,
            PromiseState::Rejected => {
                let error = value.result(scope);
                let error = deno_core::error::JsError::from_v8_exception(scope, error);
                std::task::Poll::Ready(Err(error.into()))
            }
            PromiseState::Fulfilled => {
                let result = value.result(scope);
                match deno_core::serde_v8::from_v8::<T>(scope, result) {
                    Ok(value) => std::task::Poll::Ready(Ok(value)),
                    Err(e) => std::task::Poll::Ready(Err(e.into())),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{js_value::Function, json_args, Module, Runtime, RuntimeOptions};

    #[test]
    fn test_promise() {
        let module = Module::new(
            "test.js",
            "
            export const f = () => new Promise((resolve) => resolve(42));
        ",
        );

        let mut runtime = Runtime::new(RuntimeOptions::default()).unwrap();
        let handle = runtime.load_module(&module).unwrap();

        let f: Function = runtime.get_value(Some(&handle), "f").unwrap();
        let value: Promise<usize> = f
            .call_immediate(&mut runtime, Some(&handle), &json_args!())
            .unwrap();
        let value = value.into_value(&mut runtime).unwrap();
        assert_eq!(value, 42);
    }
}
