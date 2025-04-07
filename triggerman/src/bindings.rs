use klaver::{RuntimeError, Vm};
use rquickjs::{CatchResultExt, Ctx, Function, Value};
use trigger::{BoxFuture, Task};

pub struct QuickTask {
    vm: klaver::worker::Worker,
}

impl<T> Task<T> for QuickTask
where
    T: IntoJavascript + Send + 'static,
{
    type Future<'a>
        = BoxFuture<'a, ()>
    where
        Self: 'a,
        T: 'a;

    fn call<'a>(&'a self, input: T) -> Self::Future<'a> {
        Box::pin(async move {
            klaver::async_with!(self.vm => |ctx| {

              let func = ctx.globals().get::<_, Function>("__$handler").catch(&ctx)?;
              let input = input.into_js(ctx.clone())?;

              let mut value: Value = func.call((input,)).catch(&ctx)?;
              if let Some(promise) = value.as_promise() {
                value = promise.clone().into_future::<Value>().await.catch(&ctx)?;
              }


              Ok(())
            })
            .await;
        })
    }
}

pub trait IntoJavascript {
    fn into_js<'js>(self, ctx: Ctx<'js>) -> Result<Value<'js>, RuntimeError>;
}
